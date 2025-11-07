pragma circom 2.1.6;

include "node_modules/circomlib/circuits/bitify.circom";
include "node_modules/circomlib/circuits/comparators.circom";
include "node_modules/circomlib/circuits/poseidon.circom";

// Simplified TON Event Verifier using available components
template TONEventVerifier() {
    signal input ton_tx_hash[64];      // TON transaction hash (64 bits)
    signal input recipient[64];        // Recipient address (64 bits)  
    signal input amount;               // Amount
    signal input event_id[64];         // Event ID (64 bits)
    signal input nullifier[64];        // Nullifier (64 bits)
    signal input secret;               // Secret
    
    signal output verified;            // Verification result

    // Constants
    var MAX_AMOUNT = 1000000000000;

    // Validate amount range using comparators
    signal amountValid;
    component amountCheck = LessEqThan(32);
    amountCheck.in[0] <== amount;
    amountCheck.in[1] <== MAX_AMOUNT;
    amountValid <== amountCheck.out;

    // Generate event_id using Poseidon hash
    component eventHasher = Poseidon(3);
    
    // Convert bits to field elements for hashing
    var txVal = 0;
    var recipientVal = 0;
    for (var i = 0; i < 32; i++) {
        txVal += ton_tx_hash[i] * (1 << i);
        recipientVal += recipient[i] * (1 << i);
    }
    
    eventHasher.inputs[0] <== txVal;
    eventHasher.inputs[1] <== recipientVal;
    eventHasher.inputs[2] <== amount;

    // Check event_id by comparing bits
    signal eventIdValid;
    component eventIdBits = Num2Bits(64);
    eventIdBits.in <== eventHasher.out;
    
    component eventIdCheck = BitsEqual(64);
    for (var i = 0; i < 64; i++) {
        eventIdCheck.in1[i] <== event_id[i];
        eventIdCheck.in2[i] <== eventIdBits.out[i];
    }
    eventIdValid <== eventIdCheck.out;

    // Generate nullifier using Poseidon
    component nullifierHasher = Poseidon(2);
    nullifierHasher.inputs[0] <== txVal;
    nullifierHasher.inputs[1] <== secret;

    // Check nullifier
    signal nullifierValid;
    component nullifierBits = Num2Bits(64);
    nullifierBits.in <== nullifierHasher.out;
    
    component nullifierCheck = BitsEqual(64);
    for (var i = 0; i < 64; i++) {
        nullifierCheck.in1[i] <== nullifier[i];
        nullifierCheck.in2[i] <== nullifierBits.out[i];
    }
    nullifierValid <== nullifierCheck.out;

    // Secret must be non-zero
    signal secretValid;
    component secretCheck = IsZero();
    secretCheck.in <== secret;
    secretValid <== 1 - secretCheck.out;

    // FIX: Use intermediate signals to avoid high-degree constraints
    signal step1 <== amountValid * eventIdValid;
    signal step2 <== nullifierValid * secretValid;
    
    // Final verification - now quadratic
    verified <== step1 * step2;
}

// Helper template for BitsEqual
template BitsEqual(n) {
    signal input in1[n];
    signal input in2[n];
    signal output out;
    
    component eqs[n];
    component and = MultiAND(n);
    
    for (var i = 0; i < n; i++) {
        eqs[i] = IsEqual();
        eqs[i].in[0] <== in1[i];
        eqs[i].in[1] <== in2[i];
        and.in[i] <== eqs[i].out;
    }
    
    out <== and.out;
}

// Helper template for MultiAND
template MultiAND(n) {
    signal input in[n];
    signal output out;
    
    signal intermediates[n-1];
    
    intermediates[0] <== in[0] * in[1];
    
    for (var i = 2; i < n; i++) {
        intermediates[i-1] <== intermediates[i-2] * in[i];
    }
    
    out <== intermediates[n-2];
}

component main = TONEventVerifier();