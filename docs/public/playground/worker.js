import init, { analyze, format, convert } from './pkg/gpui_rsx_wasm.js';
const ready = init();
const cache = new Map();
let latestId = 0;

function cacheKey(operation, source, strict, tailwind) {
  return JSON.stringify([operation, source, strict, tailwind]);
}

self.onmessage = async ({ data }) => {
  try {
    await ready;
    const { id, operation, source, strict, tailwind } = data;
    latestId = id;
    if (source.length > 100_000) throw new Error('Source exceeds 100 KB.');
    const key = cacheKey(operation, source, strict, tailwind);
    let result = cache.get(key);
    if (result === undefined) {
      result = operation === 'format' ? format(source)
        : operation === 'convert' ? JSON.parse(convert(source))
        : JSON.parse(analyze(source, strict, tailwind));
      if (cache.size >= 8) cache.delete(cache.keys().next().value);
      cache.set(key, result);
    }
    // Keep the cache warm, but avoid sending a result that a newer edit has superseded.
    if (id !== latestId) return;
    self.postMessage({ id, operation, result });
  } catch (error) { self.postMessage({ id: data.id, error: String(error) }); }
};
