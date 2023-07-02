import { Hono } from 'hono';
import { logger } from 'hono/logger';
import { serve } from '@hono/node-server';
import { bearerAuth } from 'hono/bearer-auth';

const app = new Hono();
const port = 3500;
const token = '';

app.use('*', logger());
app.use('/api/*', bearerAuth({ token: 'test_token' }));

app.get('/api/health', (c) =>
	c.json({
		uptime: { data: 168.44, hue: 'red' },
		version: { data: '0.2.1', hue: 'bright red' },
		engine: { data: 'docker', hue: 'yellow' },
		status: {
			ping: { data: 36, hue: 'green' },
			healthy: { data: 'yes', hue: 'cyan' },
			message: { data: 'All services running', hue: 'bright blue' },
			containers: { data: ['build', 'build/ui'], hue: 'magenta' },
		},
	})
);

console.log(`started on ${port}`);
serve({ fetch: app.fetch, port: port });
