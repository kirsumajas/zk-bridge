import { ParsedEvent, BridgeEvent, DepositEvent } from './types.js';

export class ContractParser {
  parseTransaction(tx: any, blockNumber: number): ParsedEvent | null {
    try {
      // Check if this is a transaction to our bridge contract
      const bridgeContractAddress = process.env.TON_CONTRACT_ADDRESS;
      
      if (!tx.inMsg || tx.inMsg.destination !== bridgeContractAddress) {
        return null;
      }

      // Parse the message to detect bridge events
      const bridgeEvent = this.parseBridgeMessage(tx.inMsg, tx.hash, blockNumber, tx.utime);
      if (bridgeEvent) {
        return {
          eventType: bridgeEvent.type,
          blockNumber: blockNumber,
          transactionHash: tx.hash,
          timestamp: tx.utime * 1000,
          eventData: bridgeEvent,
          rawData: tx
        };
      }

      return null;
    } catch (error) {
      console.error('Failed to parse transaction:', error);
      return null;
    }
  }

  private parseBridgeMessage(inMsg: any, txHash: string, blockNumber: number, timestamp: number): BridgeEvent | DepositEvent | null {
    if (!inMsg.msgData) {
      return null;
    }

    // Extract OP code from message data
    const opCode = this.extractOpCode(inMsg.msgData);
    
    switch (opCode) {
      case 0x4445504F: // 'DEPO' - Deposit
        return this.parseDepositEvent(inMsg, txHash, blockNumber, timestamp);
      case 0x53554343: // 'SUCC' - Settle Success
        return this.parseSettleSuccess(inMsg, txHash, blockNumber, timestamp);
      case 0x52454644: // 'REFD' - Settle Refund
        return this.parseSettleRefund(inMsg, txHash, blockNumber, timestamp);
      case 0x544D4F55: // 'TMOU' - Timeout Refund
        return this.parseTimeoutRefund(inMsg, txHash, blockNumber, timestamp);
      case 0xdec0de01: // Self-emitted deposit event
        return this.parseEmittedDepositEvent(inMsg, txHash, blockNumber, timestamp);
      default:
        return null;
    }
  }

  private extractOpCode(msgData: string): number | null {
    try {
      // OP code is typically the first 32 bits (4 bytes) of the message body
      if (msgData.length >= 8) { // 8 hex chars = 4 bytes
        return parseInt(msgData.substring(0, 8), 16);
      }
    } catch (error) {
      console.error('Failed to extract OP code:', error);
    }
    return null;
  }

  private parseDepositEvent(inMsg: any, txHash: string, blockNumber: number, timestamp: number): BridgeEvent {
    // Parse deposit event from message data
    // This would need to decode the actual message structure from your contract
    return {
      type: 'deposit',
      amount: inMsg.value || '0',
      from: inMsg.source || 'unknown',
      to: inMsg.destination,
      timestamp: timestamp * 1000,
      txHash: txHash,
      blockNumber: blockNumber
    };
  }

  private parseEmittedDepositEvent(inMsg: any, txHash: string, blockNumber: number, timestamp: number): DepositEvent | null {
    try {
      // Parse the emitted deposit event structure
      // This would need to decode the actual cell structure from your contract
      // For now, return a basic event - we can enhance this with proper cell parsing
      return {
        type: 'deposit',
        version: 1,
        workchainId: 0,
        contractHash: '', // Would extract from message
        senderHash: '',   // Would extract from message  
        recipient: '',    // Would extract from message
        amount: inMsg.value || '0',
        feeEst: '0',      // Would extract from message
        dstChainId: 1,    // Solana
        nonce: 0,         // Would extract from message
        txHash: txHash,
        blockNumber: blockNumber,
        timestamp: timestamp * 1000
      };
    } catch (error) {
      console.error('Failed to parse emitted deposit event:', error);
      return null;
    }
  }

  private parseSettleSuccess(inMsg: any, txHash: string, blockNumber: number, timestamp: number): BridgeEvent {
    return {
      type: 'settle_success',
      amount: '0', // Would extract from message
      from: inMsg.source || 'unknown',
      to: inMsg.destination,
      timestamp: timestamp * 1000,
      txHash: txHash,
      blockNumber: blockNumber
    };
  }

  private parseSettleRefund(inMsg: any, txHash: string, blockNumber: number, timestamp: number): BridgeEvent {
    return {
      type: 'settle_refund',
      amount: '0', // Would extract from message
      from: inMsg.source || 'unknown',
      to: inMsg.destination,
      timestamp: timestamp * 1000,
      txHash: txHash,
      blockNumber: blockNumber
    };
  }

  private parseTimeoutRefund(inMsg: any, txHash: string, blockNumber: number, timestamp: number): BridgeEvent {
    return {
      type: 'timeout_refund',
      amount: '0', // Would extract from message
      from: inMsg.source || 'unknown',
      to: inMsg.destination,
      timestamp: timestamp * 1000,
      txHash: txHash,
      blockNumber: blockNumber
    };
  }
}