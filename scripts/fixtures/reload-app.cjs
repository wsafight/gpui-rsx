const fs = require('node:fs');

const record = (entry) => fs.appendFileSync(process.env.GPUI_RSX_TEST_EVENTS, JSON.stringify(entry) + '\n');
process.on('SIGTERM', () => {
  record({ type: 'app-stopped', pid: process.pid });
  process.exit(0);
});
record({ type: 'app-started', pid: process.pid, readyFlag: process.env.GPUI_RSX_DEV_READY });
process.stdout.write('application output\n');
if (process.env.GPUI_RSX_DEV_READY === '1') process.stdout.write('GPUI_RSX_READY\n');
setInterval(() => {}, 1000);
