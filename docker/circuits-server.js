const express = require('express');
const cors = require('cors');

const app = express();
const port = 8080;

app.use(cors());
app.use(express.json());

app.get('/health', (req, res) => {
    res.json({ status: 'healthy', service: 'circuit' });
});

app.post('/generate-proof', (req, res) => {
    console.log('Received proof generation request:', req.body);
    
    // Mock proof generation for now
    // In production, this would run the actual Circom circuit
    const mockProof = {
        proof: "generated_proof_data",
        publicSignals: req.body.publicInputs || [],
        timestamp: Date.now()
    };
    
    res.json(mockProof);
});

app.listen(port, '0.0.0.0', () => {
    console.log(`🔐 Circuit service running on port ${port}`);
});