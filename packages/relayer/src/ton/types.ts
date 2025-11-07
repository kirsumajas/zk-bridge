export interface ParsedEvent {
  eventType: string;
  blockNumber: number;
  transactionHash: string;
  timestamp: number;
  eventData: any;
  rawData: any;
}

// TON-specific types
export interface TonTransaction {
  hash: string;
  lt: string;
  account: string;
  success: boolean;
  utime: number;
  origStatus: string;
  endStatus: string;
  totalFees: string;
  transactionType: string;
  stateUpdateOld: string;
  stateUpdateNew: string;
  inMsg: {
    source: string;
    destination: string;
    value: string;
    msgData: string;
    opCode?: number;
  };
  outMsgs: Array<{
    source: string;
    destination: string;
    value: string;
    msgData: string;
    opCode?: number;
  }>;
}

export interface TonBlock {
  workchain: number;
  shard: string;
  seqno: number;
  rootHash: string;
  fileHash: string;
  transactions: TonTransaction[];
}

// LockContract specific events
export interface DepositEvent {
  type: 'deposit';
  version: number;
  workchainId: number;
  contractHash: string;
  senderHash: string;
  recipient: string; // Solana address (256 bits)
  amount: string;
  feeEst: string;
  dstChainId: number; // Solana chain ID
  nonce: number;
  txHash: string;
  blockNumber: number;
  timestamp: number;
}

export interface BridgeEvent {
  type: 'deposit' | 'settle_success' | 'settle_refund' | 'timeout_refund';
  depositId?: string;
  amount: string;
  from: string;
  to: string;
  timestamp: number;
  txHash: string;
  blockNumber: number;
  recipientSolana?: string;
  nonce?: number;
}