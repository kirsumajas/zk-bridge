import { TonEventWatcher } from './ton/eventWatcher';

export class App {
  private eventWatcher: TonEventWatcher;

  constructor() {
    this.eventWatcher = new TonEventWatcher();
  }

  async start(): Promise<void> {
    try {
      console.log('Initializing stateless relayer...');
      
      // Check if submission manager is reachable
      const isHealthy = await this.eventWatcher.getStatus().then(s => s.submissionManagerHealth);
      if (!isHealthy) {
        console.warn('Submission manager is not reachable. Starting anyway...');
      } else {
        console.log('Submission manager is healthy');
      }
      
      console.log('Starting event watcher...');
      await this.eventWatcher.start();
    } catch (error) {
      console.error('Failed to start relayer:', error);
      process.exit(1);
    }
  }

  async stop(): Promise<void> {
    console.log('Stopping relayer...');
    this.eventWatcher.stop();
  }

  // Fix the getStatus method with proper error typing
  async getStatus() {
    try {
      return await this.eventWatcher.getStatus();
    } catch (error) {
      return {
        isRunning: false,
        currentBlock: 0,
        submissionManagerHealth: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }
}