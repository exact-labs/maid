import { Application, Router, RouterContext, upgrade } from 'https://deno.land/x/oak/mod.ts';

const app = new Application();
const router = new Router();
const port = 3500;
const token = 'test_token';

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

app.use(async (ctx, next) => {
	await next();
	console.log(`${ctx.request.method} ${ctx.request.url}`);
});

app.use(async (ctx, next) => {
	const token = ctx.request.headers.get('Authorization');
	if (token !== 'Bearer test_token') {
		ctx.response.status = 401;
		ctx.response.body = 'Unauthorized';
		return;
	}
	await next();
});

router.get('/ws/gateway', async (context: RouterContext) => {
	if (context.request.headers.get('upgrade') != 'websocket') {
		throw new Error('Request must be a WebSocket connection');
	}

	const socket = await context.upgrade();

	socket.addEventListener('open', async () => {
		console.log('A client connected.');
		socket.send(
			JSON.stringify({
				level: 'success',
				time: Date.now(),
				data: { connected: true, message: 'client connected' },
			})
		);

		await sleep(1000);

		socket.send(
			JSON.stringify({
				level: 'warning',
				time: Date.now(),
				data: { message: 'some warning idk' },
			})
		);

		await sleep(2500);

		socket.send(Deno.readFileSync('test.tgz'));

		socket.send(
			JSON.stringify({
				level: 'success',
				time: Date.now(),
				data: { done: true, message: '' },
			})
		);
	});

	socket.addEventListener('message', (event) => console.log(event.data));
});

router.get('/api/health', async (context: RouterContext) => {
	context.response.body = {
		uptime: { data: 168.44, hue: 'red' },
		version: { data: '0.2.1', hue: 'bright red' },
		engine: { data: 'docker', hue: 'yellow' },
		status: {
			ping: { data: 36, hue: 'green' },
			healthy: { data: 'yes', hue: 'cyan' },
			message: { data: 'all services running', hue: 'bright blue' },
			containers: { data: ['build', 'build/ui'], hue: 'magenta' },
		},
	};
});

app.use(router.routes());
console.log(`started on ${port}`);
await app.listen({ port: 3500 });
