import init, { analyze, format, convert } from './pkg/gpui_rsx_wasm.js';
const ready = init();
self.onmessage = async ({ data }) => {
  try {
    await ready;
    const { id, operation, source, strict, tailwind } = data;
    if (source.length > 100_000) throw new Error('Source exceeds 100 KB.');
    const result = operation === 'format' ? format(source)
      : operation === 'convert' ? JSON.parse(convert(source))
      : JSON.parse(analyze(source, strict, tailwind));
    self.postMessage({ id, operation, result });
  } catch (error) { self.postMessage({ id: data.id, error: String(error) }); }
};
