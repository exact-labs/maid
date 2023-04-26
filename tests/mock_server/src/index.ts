import { Hono } from 'hono';
import { logger } from 'hono/logger';
import { serve } from '@hono/node-server';
import { bearerAuth } from 'hono/bearer-auth';

const app = new Hono();
const port = 3500;
const token = '';

app.use('*', logger());
app.use('/api/*', bearerAuth({ prefix: '', token: 'test_token' }));

app.get('/api/health', (c) =>
	c.json({
		uptime: '168.44d',
		version: '0.2.1',
		engine: 'docker',
		status: {
			healthy: true,
			ping: 36,
			message: 'All services running',
			containers: ['build', 'build/ui'],
		},
	})
);

console.log(`started on ${port}`);
serve({ fetch: app.fetch, port: port });
