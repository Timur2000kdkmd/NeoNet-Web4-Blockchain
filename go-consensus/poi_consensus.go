package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/rand"
	"net/http"
	"sync"
	"time"
)

type AIProof struct {
	ModelHash      string  `json:"model_hash"`
	GradientHash   string  `json:"gradient_hash"`
	AccuracyScore  float64 `json:"accuracy_score"`
	LossValue      float64 `json:"loss_value"`
	TrainingRounds int     `json:"training_rounds"`
	ValidatorID    string  `json:"validator_id"`
	Timestamp      int64   `json:"timestamp"`
	Signature      string  `json:"signature"`
}

type PoIBlock struct {
	Index        int         `json:"index"`
	Timestamp    int64       `json:"timestamp"`
	Transactions []string    `json:"transactions"`
	AIProof      AIProof     `json:"ai_proof"`
	PrevHash     string      `json:"prev_hash"`
	Hash         string      `json:"hash"`
	Proposer     string      `json:"proposer"`
}

type AIValidator struct {
	ID              string  `json:"id"`
	Stake           float64 `json:"stake"`
	ComputePower    float64 `json:"compute_power"`
	Reputation      float64 `json:"reputation"`
	BlocksValidated int     `json:"blocks_validated"`
	TotalAccuracy   float64 `json:"total_accuracy"`
	IsActive        bool    `json:"is_active"`
}

type PoIConsensus struct {
	validators     map[string]*AIValidator
	pendingProofs  []AIProof
	currentRound   int
	minAccuracy    float64
	minValidators  int
	aiServiceURL   string
	mu             sync.RWMutex
}

func NewPoIConsensus(aiServiceURL string) *PoIConsensus {
	return &PoIConsensus{
		validators:    make(map[string]*AIValidator),
		pendingProofs: make([]AIProof, 0),
		currentRound:  0,
		minAccuracy:   0.7,
		minValidators: 3,
		aiServiceURL:  aiServiceURL,
	}
}

func (poi *PoIConsensus) RegisterValidator(id string, stake, computePower float64) bool {
	poi.mu.Lock()
	defer poi.mu.Unlock()

	if _, exists := poi.validators[id]; exists {
		return false
	}

	poi.validators[id] = &AIValidator{
		ID:              id,
		Stake:           stake,
		ComputePower:    computePower,
		Reputation:      1.0,
		BlocksValidated: 0,
		TotalAccuracy:   0.0,
		IsActive:        true,
	}
	return true
}

func (poi *PoIConsensus) SubmitAIProof(validatorID string, modelWeights, gradients []float64, accuracy, loss float64, rounds int) *AIProof {
	poi.mu.Lock()
	defer poi.mu.Unlock()

	validator, exists := poi.validators[validatorID]
	if !exists || !validator.IsActive {
		return nil
	}

	if accuracy < poi.minAccuracy {
		return nil
	}

	modelHash := hashFloatSlice(modelWeights)
	gradientHash := hashFloatSlice(gradients)
	signature := signProof(validatorID, modelHash, gradientHash)

	proof := AIProof{
		ModelHash:      modelHash,
		GradientHash:   gradientHash,
		AccuracyScore:  accuracy,
		LossValue:      loss,
		TrainingRounds: rounds,
		ValidatorID:    validatorID,
		Timestamp:      time.Now().Unix(),
		Signature:      signature,
	}

	poi.pendingProofs = append(poi.pendingProofs, proof)
	return &proof
}

func (poi *PoIConsensus) VerifyAIProof(proof *AIProof) (bool, string) {
	poi.mu.RLock()
	defer poi.mu.RUnlock()

	validator, exists := poi.validators[proof.ValidatorID]
	if !exists {
		return false, "Unknown validator"
	}

	if !validator.IsActive {
		return false, "Validator not active"
	}

	if proof.AccuracyScore < poi.minAccuracy {
		return false, fmt.Sprintf("Accuracy %.2f below minimum %.2f", proof.AccuracyScore, poi.minAccuracy)
	}

	if proof.TrainingRounds < 1 {
		return false, "Invalid training rounds"
	}

	expectedSig := signProof(proof.ValidatorID, proof.ModelHash, proof.GradientHash)
	if proof.Signature != expectedSig {
		return false, "Invalid signature"
	}

	return true, "Valid proof"
}

func (poi *PoIConsensus) VerifyModelGradient(data map[string]interface{}, proof *AIProof) (bool, float64) {
	valid, _ := poi.VerifyAIProof(proof)
	if !valid {
		return false, 0.0
	}

	confidence := poi.calculateConfidence(proof)

	if confidence > 0.5 {
		poi.mu.Lock()
		if validator, exists := poi.validators[proof.ValidatorID]; exists {
			validator.BlocksValidated++
			validator.TotalAccuracy += proof.AccuracyScore
			validator.Reputation = min(2.0, validator.Reputation*1.01)
		}
		poi.mu.Unlock()
	}

	return valid, confidence
}

func (poi *PoIConsensus) SelectBlockProposer() string {
	poi.mu.RLock()
	defer poi.mu.RUnlock()

	if len(poi.validators) < poi.minValidators {
		return ""
	}

	var totalWeight float64
	weights := make(map[string]float64)

	for id, v := range poi.validators {
		if !v.IsActive {
			continue
		}
		weight := v.Stake*0.4 + v.ComputePower*0.3 + v.Reputation*0.3
		weights[id] = weight
		totalWeight += weight
	}

	if totalWeight == 0 {
		return ""
	}

	r := rand.Float64() * totalWeight
	var cumulative float64
	for id, weight := range weights {
		cumulative += weight
		if r <= cumulative {
			return id
		}
	}

	return ""
}

func (poi *PoIConsensus) CreatePoIBlock(prevBlock *PoIBlock, transactions []string, proposerID string, proof *AIProof) *PoIBlock {
	valid, _ := poi.VerifyModelGradient(nil, proof)
	if !valid {
		return nil
	}

	block := &PoIBlock{
		Index:        prevBlock.Index + 1,
		Timestamp:    time.Now().Unix(),
		Transactions: transactions,
		AIProof:      *proof,
		PrevHash:     prevBlock.Hash,
		Proposer:     proposerID,
	}

	block.Hash = computePoIBlockHash(block)
	poi.currentRound++

	return block
}

func (poi *PoIConsensus) ValidateBlockWithAI(block *PoIBlock) (bool, float64, error) {
	if poi.aiServiceURL == "" {
		return poi.localValidation(block)
	}

	reqBody, _ := json.Marshal(map[string]interface{}{
		"block_hash":    block.Hash,
		"proposer":      block.Proposer,
		"ai_proof":      block.AIProof,
		"tx_count":      len(block.Transactions),
	})

	resp, err := http.Post(
		poi.aiServiceURL+"/ai/validate_block",
		"application/json",
		nil,
	)
	if err != nil {
		return poi.localValidation(block)
	}
	defer resp.Body.Close()

	var result struct {
		Valid      bool    `json:"valid"`
		Confidence float64 `json:"confidence"`
	}
	json.NewDecoder(resp.Body).Decode(&result)

	return result.Valid, result.Confidence, nil
}

func (poi *PoIConsensus) localValidation(block *PoIBlock) (bool, float64, error) {
	valid, confidence := poi.VerifyModelGradient(nil, &block.AIProof)
	return valid, confidence, nil
}

func (poi *PoIConsensus) FederatedAggregate(gradients [][]float64) []float64 {
	if len(gradients) == 0 {
		return nil
	}

	length := len(gradients[0])
	result := make([]float64, length)

	for i := 0; i < length; i++ {
		sum := 0.0
		for _, g := range gradients {
			if i < len(g) {
				sum += g[i]
			}
		}
		result[i] = sum / float64(len(gradients))
	}

	return result
}

func (poi *PoIConsensus) GetNetworkStats() map[string]interface{} {
	poi.mu.RLock()
	defer poi.mu.RUnlock()

	var totalStake, totalCompute float64
	var totalBlocks int
	activeCount := 0

	for _, v := range poi.validators {
		totalStake += v.Stake
		totalCompute += v.ComputePower
		totalBlocks += v.BlocksValidated
		if v.IsActive {
			activeCount++
		}
	}

	return map[string]interface{}{
		"total_validators":   len(poi.validators),
		"active_validators":  activeCount,
		"total_stake":        totalStake,
		"total_compute":      totalCompute,
		"total_blocks":       totalBlocks,
		"current_round":      poi.currentRound,
		"pending_proofs":     len(poi.pendingProofs),
		"min_accuracy":       poi.minAccuracy,
	}
}

func (poi *PoIConsensus) GetValidatorStats(id string) *AIValidator {
	poi.mu.RLock()
	defer poi.mu.RUnlock()
	return poi.validators[id]
}

func (poi *PoIConsensus) calculateConfidence(proof *AIProof) float64 {
	accuracyWeight := proof.AccuracyScore * 0.5
	lossWeight := max(0, 1-proof.LossValue) * 0.3
	roundsWeight := min(1.0, float64(proof.TrainingRounds)/100) * 0.2
	return accuracyWeight + lossWeight + roundsWeight
}

func hashFloatSlice(data []float64) string {
	bytes, _ := json.Marshal(data)
	hash := sha256.Sum256(bytes)
	return hex.EncodeToString(hash[:])
}

func signProof(validatorID, modelHash, gradientHash string) string {
	data := fmt.Sprintf("%s:%s:%s", validatorID, modelHash, gradientHash)
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

func computePoIBlockHash(block *PoIBlock) string {
	data, _ := json.Marshal(map[string]interface{}{
		"index":        block.Index,
		"timestamp":    block.Timestamp,
		"transactions": block.Transactions,
		"ai_proof":     block.AIProof,
		"prev_hash":    block.PrevHash,
		"proposer":     block.Proposer,
	})
	hash := sha256.Sum256(data)
	return hex.EncodeToString(hash[:])
}

func min(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

func max(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}
