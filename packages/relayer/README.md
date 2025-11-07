# TON Bridge Listener

Event listener service for TON-Solana bridge that monitors TON blockchain for deposit events and generates ZK proofs.

## Setup

1. Copy `.env.example` to `.env` and configure your environment variables
2. Run `npm install` to install dependencies
3. Start database and Redis: `docker-compose up db redis`
4. Initialize database: `npm run db:init`
5. Start development server: `npm run dev`

## Development

- `npm run dev` - Start development server with hot reload
- `npm run build` - Build TypeScript to JavaScript
- `npm run start` - Run production build