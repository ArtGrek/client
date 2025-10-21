//playwright_ws.js
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const WebSocket = require('ws');

async function main() {
	let browser, page, wss, fileStream;
	console.log("[STEP] Set signals");
	process.on('SIGINT', shutdown);
	process.on('SIGTERM', shutdown);
	process.on('uncaughtException', shutdown);
	process.on('unhandledRejection', shutdown);
	async function shutdown(err) {
		try {
			if (err) console.error('\t[ERROR]', err);
			console.log('[STEP] Cleanup page, browser, WebSocket, fileStream...');
			try {
				if (page && !page.isClosed()) {
					await page.close();
					console.log('\t[INFO] Page closed');
				}
			} catch (e) { console.error('\t[WARN] Failed to close page:', e.message); }
			try {
				if (browser && browser.isConnected()) {
					await browser.close();
					console.log('\t[INFO] Browser session closed');
				}
			} catch (e) { console.error('\t[WARN] Failed to close browser session:', e.message); }
			try {
				if (wss) wss.close(() => console.log('\t[INFO] WebSocket server closed'));
			} catch (e) { console.error('\t[WARN] Failed to close WebSocket server:', e.message); }
			try {
				if (fileStream) {
					fileStream.end();
					console.log('\t[INFO] File stream closed');
				}
			} catch (e) { console.error('\t[WARN] Failed to close fileStream:', e.message); }
		} catch (e) {
			console.error('\t[ERROR]', e);
		} finally {
			console.log('[STEP] Process exit.');
			process.exit(0);
		}
	}
	try {
		console.log("[STEP] Run script");	
		if (process.argv.length < 5) {
			console.error('\t[ERROR] use: node ./playwright/<provider_name>.js <launch_url> <temp_data_dir> <transactions_file> <ws_port>');
			process.exit(1);
		}
		const launchUrl = process.argv[2];
		const tempDataDir = process.argv[3];
		const transactionsFile = process.argv[4];
		const wsPort = Number(process.argv[5]);
		console.log("[STEP] Create transactions file and folders");	
		fs.mkdirSync(tempDataDir, { recursive: true });
		fileStream = fs.createWriteStream(transactionsFile, { flags: 'a', encoding: 'utf8' });
		console.log("[STEP] Create script log file");	
		const LOG_FILE = path.join(tempDataDir || ".", "ws_log.txt");
		function logToFile(message) {fs.appendFileSync(LOG_FILE, message + "\n", "utf8");}
		let log_msg;
		console.log("[STEP] Run and connect to Chrome");
		browser = await chromium.connectOverCDP('http://localhost:9222');
		const defaultContext = browser.contexts()[0];
		page = defaultContext.pages()[0] || await defaultContext.newPage();
		console.log("[STEP] Set listener for init...");
		let lastInitResponse = null;
		const requests = new Map();
		page.on('request', async request => {
			if (request.method() === 'POST' && request.url().includes('/platform/public/init')) {
				requests.set(request, true);
				console.log("\t[INFO] /init captured");
			}
		});

		page.on('response', async response => {
			const request = response.request();
			if (request.method() === 'POST' && request.url().includes('/platform/public/init')) {
				if (requests.has(request)) {
					let responseBody = '';
					try {
						responseBody = await response.text();
					} catch {
						responseBody = '[UNAVAILABLE]';
					}
					const postData = request.postData() || '';
					fileStream.write('{"in":' + postData + ',"out":' + responseBody + '},\n');
					requests.delete(request);
					lastInitResponse = responseBody;
					console.log("\t[INFO] Save init transaction");
				}
			}
		});
		console.log("[STEP] Run WebSocket on port: ", wsPort);
		wss = new WebSocket.Server({ port: wsPort });
		wss.on('connection', ws => {
			ws.on('message', async (log_msg) => {
				try {
					logToFile('[WS] received: ' + log_msg.toString());
					const req = JSON.parse(log_msg.toString());
					if (req.type === "shutdown") {
						ws.send("");
						logToFile('[WS] received shutdown command');
						console.log("\t[INFO] Shutdown command received via WS");
						shutdown(); 
						return;
					}
					if (req.type === "get_last_state") {
						let toSend = "";
						if (lastInitResponse == null) {
							toSend = "";
						} else if (typeof lastInitResponse === "string") {
							toSend = lastInitResponse;
						} else {
							toSend = JSON.stringify(lastInitResponse);
						}
						ws.send(toSend);
						logToFile('[WS] sent: ' + toSend);
						return;
					}
					if (req.type === "api") {
						try {
							const result = await page.evaluate(async ({ url, body, headers }) => {
								const resp = await window.fetch(url, {
									method: "POST",
									headers: Object.assign({
										"accept": "application/json, text/plain, */*",
										"content-type": "application/json"
									}, headers || {}),
									body: JSON.stringify(body)
								});
								return await resp.text();
							}, req.data);
							ws.send(result);
							logToFile('[WS] sent: ' + result);
						} catch (e) {
							ws.send(JSON.stringify({ status: "error", detail: e.message }));
							logToFile('[WS] sent: ' + JSON.stringify({ status: "error", detail: e.message }));
						}
						return;
					}
					ws.send(JSON.stringify({ error: "unknown type" }));
					logToFile('[WS] sent: ' + JSON.stringify({ error: "unknown type" }));
				} catch (e) {
					ws.send(JSON.stringify({ error: e.message || "error" }));
					logToFile('[WS] sent: ' + JSON.stringify({ error: e.message || "error" }));
				}
			});
		});
		
		console.log("[STEP] Load page: ", launchUrl);
		try {
			await page.goto(launchUrl, {waitUntil: 'domcontentloaded', timeout: 15000});
		} catch (e) {
			console.error("\t[ERROR] Loading: ", e.message);
		}
		try {
			await page.waitForLoadState('networkidle', { timeout: 15000 });
		} catch (e) {
			console.error("\t[ERROR] Loaded: ", e.message);
		}
		console.log("[STEP] Save page: ", launchUrl);
		const iframeHtml = await page.content();
		fs.writeFileSync(path.join(tempDataDir, 'iframe.html'), iframeHtml, 'utf8');
		console.log("[STEP] Click to START");
		try {
			await page.waitForSelector('div.start-button', { state: 'visible', timeout: 15000 });
			await page.click('div.start-button');
			console.log("\t[INFO] Click to START: OK");
		} catch (e) {
			console.error("\t[ERROR] Click to START: ", e.message);
		}
	} catch (e) {shutdown(e)}
	console.log("[STEP] Ready. Press Ctrl+C for exit.");
}

main();
