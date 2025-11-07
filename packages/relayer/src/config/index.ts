import { CONSTANTS } from './constants.js';

export interface Config {
  rpcUrl: string;
  submissionManagerUrl: string;
  pollInterval: number;
}

export const config: Config = {
  rpcUrl: process.env.TON_RPC_URL || CONSTANTS.DEFAULT_RPC_URL,
  submissionManagerUrl: process.env.SUBMISSION_MANAGER_URL || CONSTANTS.SUBMISSION_MANAGER_URL,
  pollInterval: parseInt(process.env.TON_POLL_INTERVAL_MS || CONSTANTS.POLL_INTERVAL.toString())
};