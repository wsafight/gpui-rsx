const fs = require('node:fs');

const record = (entry) => fs.appendFileSync(process.env.GPUI_RSX_TEST_EVENTS, JSON.stringify(entry) + '\n');
process.on('exit', () => record({ type: 'build-finished', pid: process.pid }));
const source = fs.readFileSync(process.env.GPUI_RSX_TEST_SOURCE, 'utf8');
if (source.includes('block')) {
  process.on('SIGTERM', () => {
    record({ type: 'build-stopped', pid: process.pid });
    process.exit(0);
  });
}
record({ type: 'build', pid: process.pid, args: process.argv.slice(2) });
if (source.includes('fail')) {
  process.stderr.write('fixture compilation failed\n');
  process.exit(1);
}
if (source.includes('block')) {
  setInterval(() => {}, 1000);
} else {
  process.stdout.write('non-json cargo output\n');
  process.stdout.write(JSON.stringify({ reason: 'compiler-artifact', target: { name: 'fixture-app' },
    executable: process.env.GPUI_RSX_TEST_APP }) + '\n');
  process.stdout.write(JSON.stringify({ reason: 'compiler-artifact', target: { name: 'unrelated-target' },
    executable: '/not/the/requested/binary' }) + '\n');
}
