import { App } from './app';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

const app = new App();

// Handle graceful shutdown
process.on('SIGINT', async () => {
  console.log('Received SIGINT, shutting down gracefully...');
  await app.stop();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('Received SIGTERM, shutting down gracefully...');
  await app.stop();
  process.exit(0);
});

// Simple status endpoint for health checks
if (process.env.NODE_ENV !== 'production') {
  const http = require('http');
  const server = http.createServer(async (req: any, res: any) => {
    if (req.url === '/health') {
      try {
        const status = await app.getStatus();
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(status, null, 2));
      } catch (error: any) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: 'Failed to get status' }));
      }
    } else {
      res.writeHead(404);
      res.end('Not found');
    }
  });
  server.listen(3002, () => {
    console.log('Relayer health check server running on port 3002');
  });
}

// Start the application
app.start().catch(console.error);