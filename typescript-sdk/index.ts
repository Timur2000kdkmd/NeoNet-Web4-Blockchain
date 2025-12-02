import axios from 'axios';

export class NeoAIService {
    base: string;
    constructor(base='http://localhost:8000') { this.base = base; }

    async registerMiner(cpu_cores: number, gpu_memory_mb: number, endpoint: string) {
        const res = await axios.post(`${this.base}/register_miner`, { cpu_cores, gpu_memory_mb, endpoint });
        return res.data;
    }

    async submitTask(model_id: string, payload_ref: string) {
        const res = await axios.post(`${this.base}/submit_task`, { model_id, payload_ref });
        return res.data;
    }

    async taskStatus(task_id: string) {
        const res = await axios.get(`${this.base}/task_status/${task_id}`);
        return res.data;
    }
}
