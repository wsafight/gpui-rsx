import init, { analyze_with_theme, format, convert_with_options } from './pkg/gpui_rsx_wasm.js';
const ready = init();
const cache = new Map();
let latestId = 0;

self.onmessage = async ({ data }) => {
  try {
    if (!data || !Number.isSafeInteger(data.id) || !['analyze', 'format', 'convert'].includes(data.operation)
      || typeof data.source !== 'string' || (data.theme !== undefined && typeof data.theme !== 'string')
      || (data.strict !== undefined && typeof data.strict !== 'boolean') || (data.tailwind !== undefined && typeof data.tailwind !== 'boolean')) {
      throw new Error('Invalid playground request.');
    }
    const { id, operation, source, strict = false, tailwind = false, theme = '' } = data;
    latestId = id;
    if (source.length > 100_000 || theme.length > 20_000) throw new Error('Source or theme exceeds the playground limit.');
    await ready;
    const key = JSON.stringify([operation, source, strict, tailwind, theme]);
    let result = cache.get(key);
    if (result === undefined) {
      if (operation === 'convert') {
        const settings = JSON.parse(analyze_with_theme('<div/>', false, tailwind, theme));
        if (settings.theme_diagnostics.length) throw new Error(settings.theme_diagnostics.join('\n'));
        result = JSON.parse(convert_with_options(source, settings.tailwind));
      } else {
        result = operation === 'format' ? format(source)
          : JSON.parse(analyze_with_theme(source, strict, tailwind, theme));
      }
      if (cache.size >= 8) cache.delete(cache.keys().next().value);
      cache.set(key, result);
    }
    if (id !== latestId) return;
    self.postMessage({ id, operation, result });
  } catch (error) { self.postMessage({ id: data?.id, error: String(error) }); }
};
