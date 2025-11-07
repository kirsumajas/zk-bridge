import { ContractParser } from './contractParser';
import { ParsedEvent } from './types';
import { EventForwarder } from '../services/eventForwarder';
import { config } from '../config';
import axios from 'axios';

export class TonEventWatcher {
  private parser: ContractParser;
  private forwarder: EventForwarder;
  private isRunning: boolean = false;
  private currentBlock: number = 0;
  private lastRequestTime: number = 0;
  private readonly MIN_REQUEST_INTERVAL = 1000; // 1 second between requests

  constructor() {
    this.parser = new ContractParser();
    this.forwarder = new EventForwarder();
  }

  async start(): Promise<void> {
    this.isRunning = true;
    console.log('Starting TON event watcher...');
    console.log(`Watching contract: ${process.env.TON_CONTRACT_ADDRESS}`);

    // Get initial block number from TON blockchain
    this.currentBlock = await this.getCurrentBlockNumber();
    console.log(`Starting from block: ${this.currentBlock}`);

    // Start the polling loop
    this.pollLoop();
  }

  private async pollLoop(): Promise<void> {
    while (this.isRunning) {
      try {
        await this.processNewBlocks();
      } catch (error) {
        console.error('Error processing blocks:', error);
      }

      // Wait before next poll
      await this.delay(config.pollInterval);
    }
  }

  stop(): void {
    this.isRunning = false;
    console.log('Stopping TON event watcher...');
  }

  private async processNewBlocks(): Promise<void> {
    const latestBlock = await this.getCurrentBlockNumber();
    
    if (latestBlock <= this.currentBlock) {
      console.log(`No new blocks. Current: ${this.currentBlock}, Latest: ${latestBlock}`);
      return;
    }

    // Process only a few blocks at a time to avoid rate limits
    const blocksToProcess = Math.min(latestBlock - this.currentBlock, 3);
    const endBlock = this.currentBlock + blocksToProcess;

    console.log(`Processing blocks from ${this.currentBlock + 1} to ${endBlock}`);

    for (let blockNumber = this.currentBlock + 1; blockNumber <= endBlock; blockNumber++) {
      try {
        await this.throttleRequest();
        
        const transactions = await this.getBlockTransactions(blockNumber);
        const parsedEvents = this.parseTransactions(transactions, blockNumber);

        if (parsedEvents.length > 0) {
          console.log(`Found ${parsedEvents.length} bridge events in block ${blockNumber}`);
          for (const event of parsedEvents) {
            console.log(`  - ${event.eventType} from ${event.eventData.from}`);
          }
          
          const success = await this.forwarder.forwardEventsWithRetry(parsedEvents);
          
          if (!success) {
            console.error(`Failed to forward events from block ${blockNumber}`);
          }
        } else {
          console.log(`No bridge events found in block ${blockNumber}`);
        }

        this.currentBlock = blockNumber;
      } catch (error: any) {
        if (error.response?.status === 429) {
          console.log(`Rate limit hit at block ${blockNumber}, waiting 5 seconds...`);
          await this.delay(5000);
          continue; // Retry the same block
        }
        console.error(`Error processing block ${blockNumber}:`, error.message);
        // Continue with next block even if one fails
      }
    }

    console.log(`Processed up to block ${this.currentBlock}`);
  }

  private async throttleRequest(): Promise<void> {
    const now = Date.now();
    const timeSinceLastRequest = now - this.lastRequestTime;
    
    if (timeSinceLastRequest < this.MIN_REQUEST_INTERVAL) {
      await this.delay(this.MIN_REQUEST_INTERVAL - timeSinceLastRequest);
    }
    
    this.lastRequestTime = Date.now();
  }

  private async getCurrentBlockNumber(): Promise<number> {
    try {
      await this.throttleRequest();
      
      const response = await axios.post(config.rpcUrl, {
        jsonrpc: '2.0',
        id: 1,
        method: 'getMasterchainInfo',
        params: {}
      });

      return response.data.result.last.seqno;
    } catch (error: any) {
      if (error.response?.status === 429) {
        console.log('Rate limit hit getting block number, waiting 5 seconds...');
        await this.delay(5000);
        return this.getCurrentBlockNumber(); // Retry
      }
      console.error('Failed to get current block number:', error.message);
      // Fallback: increment slowly for testing
      return this.currentBlock + 1;
    }
  }

  private async getBlockTransactions(blockNumber: number): Promise<any[]> {
    try {
      // Try a simpler method first - get transactions by address (more efficient)
      const contractAddress = process.env.TON_CONTRACT_ADDRESS;
      if (contractAddress) {
        const transactions = await this.getTransactionsByAddress(contractAddress, blockNumber);
        if (transactions.length > 0) {
          return transactions;
        }
      }

      // Fallback to block transactions if no contract-specific transactions found
      const response = await axios.post(config.rpcUrl, {
        jsonrpc: '2.0',
        id: 1,
        method: 'getBlockTransactions',
        params: {
          workchain: -1, // masterchain
          shard: '-9223372036854775808',
          seqno: blockNumber,
          count: 10 // Reduced count to avoid rate limits
        }
      });

      return response.data.result.transactions || [];
    } catch (error: any) {
      if (error.response?.status === 429) {
        throw error; // Let the caller handle rate limits
      }
      console.error(`Failed to get transactions for block ${blockNumber}:`, error.message);
      return [];
    }
  }

  private async getTransactionsByAddress(address: string, blockNumber: number): Promise<any[]> {
    try {
      // Get transactions for our specific contract address (more efficient)
      const response = await axios.post(config.rpcUrl, {
        jsonrpc: '2.0',
        id: 1,
        method: 'getTransactions',
        params: {
          address: address,
          limit: 10,
          archival: true
        }
      });

      // Filter transactions by block number
      const transactions = response.data.result || [];
      return transactions.filter((tx: any) => {
        // Note: This might not be the exact block number in the response
        // We'll need to adapt based on the actual response structure
        return true; // For now, return all transactions for the address
      });
    } catch (error: any) {
      if (error.response?.status === 429) {
        throw error;
      }
      console.error(`Failed to get transactions for address ${address}:`, error.message);
      return [];
    }
  }

  private parseTransactions(transactions: any[], blockNumber: number): ParsedEvent[] {
    const parsedEvents: ParsedEvent[] = [];
    
    for (const tx of transactions) {
      const parsed = this.parser.parseTransaction(tx, blockNumber);
      if (parsed) {
        parsedEvents.push(parsed);
      }
    }

    return parsedEvents;
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  async getStatus() {
    const health = await this.forwarder.healthCheck();
    return {
      isRunning: this.isRunning,
      currentBlock: this.currentBlock,
      submissionManagerHealth: health,
      contractAddress: process.env.TON_CONTRACT_ADDRESS
    };
  }
}