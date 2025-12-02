const { expect } = require('chai');
const { ethers } = require('hardhat');

describe('NeoNet Full Integration', function() {
  let token, stake, oracle;
  let deployer, user1, user2, relayer;

  before(async function() {
    [deployer, user1, user2, relayer] = await ethers.getSigners();
    const NeoToken = await ethers.getContractFactory('NeoToken');
    token = await NeoToken.deploy(ethers.utils.parseEther('1000000'));
    await token.deployed();
    const Stake = await ethers.getContractFactory('StakeAndVote');
    stake = await Stake.deploy(token.address);
    await stake.deployed();
    const Oracle = await ethers.getContractFactory('Oracle');
    oracle = await Oracle.deploy();
    await oracle.deployed();
    await stake.setOracle(oracle.address);
    // register signers in oracle (user1, user2)
    await oracle.addSigner(user1.address);
    await oracle.addSigner(user2.address);
    // register relayer
    await oracle.addRelayer(relayer.address);
    // transfer tokens and approve staking for user1
    await token.transfer(user1.address, ethers.utils.parseEther('1000'));
    await token.connect(user1).approve(stake.address, ethers.utils.parseEther('1000'));
  });

  it('user can stake and create proposal and vote', async function() {
    await stake.connect(user1).stake(ethers.utils.parseEther('100'));
    const tx = await stake.connect(user1).createProposal('QmFakeIpfs', 10);
    const receipt = await tx.wait();
    const event = receipt.events.find(e => e.event === 'ProposalCreated');
    const pid = event.args[0];
    // vote
    await stake.connect(user1).vote(pid, true);
    const p = await stake.proposals(pid);
    expect(p.forVotes).to.be.gt(0);
  });

  it('oracle can accept aggregated report and emit event', async function() {
    // create fake report hash and signatures using signers
    const proposalId = ethers.utils.formatBytes32String('1');
    const reportId = ethers.utils.formatBytes32String('r1');
    const resultHash = ethers.utils.formatBytes32String('ok');
    const msgHash = ethers.utils.keccak256(ethers.utils.defaultAbiCoder.encode(['bytes32','bytes32','bytes32'], [proposalId, reportId, resultHash]));
    // sign the msgHash with user1 and user2
    const sig1 = await user1.signMessage(ethers.utils.arrayify(msgHash));
    const sig2 = await user2.signMessage(ethers.utils.arrayify(msgHash));
    const sigs = [sig1, sig2];
    await expect(oracle.connect(relayer).submitReport(proposalId, reportId, resultHash, sigs, 2))
      .to.emit(oracle, 'ReportSubmitted');
  });
});
