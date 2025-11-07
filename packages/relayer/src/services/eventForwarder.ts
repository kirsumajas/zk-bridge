import axios from 'axios';
import { ParsedEvent } from '../ton/types';
import { config } from '../config';
import { CONSTANTS } from '../config/constants';

export class EventForwarder {
  private readonly submissionManagerUrl: string;

  constructor() {
    this.submissionManagerUrl = config.submissionManagerUrl;
  }

  async forwardEvents(events: ParsedEvent[]): Promise<boolean> {
    if (events.length === 0) return true;

    try {
      const response = await axios.post(
        `${this.submissionManagerUrl}/events`,
        { events },
        { 
          timeout: 10000,
          headers: {
            'Content-Type': 'application/json'
          }
        }
      );

      return response.status === 200;
    } catch (error: any) {
      console.error('Failed to forward events to submission manager:', error.message);
      return false;
    }
  }

  async forwardEventsWithRetry(events: ParsedEvent[]): Promise<boolean> {
    for (let attempt = 1; attempt <= CONSTANTS.MAX_RETRIES; attempt++) {
      console.log(`Forwarding ${events.length} events to submission manager (attempt ${attempt})`);
      
      const success = await this.forwardEvents(events);
      if (success) {
        console.log(`Successfully forwarded ${events.length} events`);
        return true;
      }

      if (attempt < CONSTANTS.MAX_RETRIES) {
        console.log(`Retrying in ${CONSTANTS.RETRY_DELAY * attempt}ms...`);
        await this.delay(CONSTANTS.RETRY_DELAY * attempt);
      }
    }
    
    console.error(`Failed to forward events after ${CONSTANTS.MAX_RETRIES} attempts`);
    return false;
  }

  async healthCheck(): Promise<boolean> {
    try {
      const response = await axios.get(`${this.submissionManagerUrl}/health`, { timeout: 5000 });
      return response.status === 200;
    } catch (error) {
      return false;
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}