const numericList = (value) => {
    if (!Array.isArray(value)) {
        return [];
    }
    return value.map((entry) => (typeof entry === 'number' ? entry : Number(entry)));
};

const ensureArray = (value) => (Array.isArray(value) ? value : []);

const duplicates = (values) => {
    const seen = new Set();
    const dup = new Set();
    values.forEach((val) => {
        const key = JSON.stringify(val);
        if (seen.has(key)) {
            dup.add(key);
        } else {
            seen.add(key);
        }
    });
    return Array.from(dup).map((entry) => JSON.parse(entry));
};

export const builtins = {
    count: (list) => ensureArray(list).length,
    sum: (list) => numericList(list).reduce((acc, val) => acc + val, 0),
    max: (list) => {
        const values = numericList(list);
        return values.length === 0 ? undefined : Math.max(...values);
    },
    min: (list) => {
        const values = numericList(list);
        return values.length === 0 ? undefined : Math.min(...values);
    },
    product: (list) => {
        const values = numericList(list);
        return values.reduce((acc, val) => acc * val, values.length ? 1 : 0);
    },
    mean: (list) => {
        const values = numericList(list);
        return values.length === 0
            ? undefined
            : values.reduce((acc, val) => acc + val, 0) / values.length;
    },
    median: (list) => {
        const values = numericList(list).sort((a, b) => a - b);
        const len = values.length;
        if (len === 0) return undefined;
        const mid = Math.floor(len / 2);
        return len % 2 === 0 ? (values[mid - 1] + values[mid]) / 2 : values[mid];
    },
    stddev: (list) => {
        const values = numericList(list);
        if (values.length === 0) return undefined;
        const mean = values.reduce((acc, val) => acc + val, 0) / values.length;
        const variance =
            values.reduce((acc, val) => acc + (val - mean) ** 2, 0) / values.length;
        return Math.sqrt(variance);
    },
    mode: (list) => {
        const values = numericList(list);
        const counts = new Map();
        values.forEach((v) => counts.set(v, (counts.get(v) || 0) + 1));
        let maxCount = 0;
        let mode = undefined;
        counts.forEach((count, value) => {
            if (count > maxCount) {
                maxCount = count;
                mode = value;
            }
        });
        return mode;
    },
    any: (list) => (Array.isArray(list) ? list.some(Boolean) : !!list),
    all: (list) => (Array.isArray(list) ? list.every(Boolean) : !!list),
    flatten: (list) => (Array.isArray(list) ? list.flat(Infinity) : list),
    distinctValues: (list) =>
        Array.isArray(list) ? Array.from(new Set(list.map((v) => JSON.stringify(v)))).map((s) => JSON.parse(s)) : [],
    duplicateValues: (list) => (Array.isArray(list) ? duplicates(list) : []),
    reverse: (list) => (Array.isArray(list) ? [...list].reverse() : list),
    append: (...lists) => lists.flatMap((entry) => (Array.isArray(entry) ? entry : [entry])),
    concatenate: (...lists) => lists.flatMap((entry) => (Array.isArray(entry) ? entry : [entry])),
    union: (...lists) => Array.from(new Set(lists.flat())),
    sublist: (list, start, length) => {
        const arr = ensureArray(list);
        const from = Math.max(0, (Number(start) || 1) - 1);
        if (length === undefined) {
            return arr.slice(from);
        }
        return arr.slice(from, from + Number(length));
    },
    insertBefore: (list, position, value) => {
        const arr = ensureArray(list);
        const idx = Math.max(0, Math.min(arr.length, (Number(position) || 1) - 1));
        const copy = [...arr];
        copy.splice(idx, 0, value);
        return copy;
    },
    remove: (list, position) => {
        const arr = ensureArray(list);
        const idx = (Number(position) || 1) - 1;
        return arr.filter((_, i) => i !== idx);
    },
    indexOf: (list, value) => {
        const arr = ensureArray(list);
        return arr
            .map((v, idx) => ({ v, idx }))
            .filter(({ v }) => v === value)
            .map(({ idx }) => idx + 1);
    },
    partition: (list, size) => {
        const arr = ensureArray(list);
        const chunk = Math.max(0, Number(size) || 0);
        if (chunk <= 0) {
            return [arr.slice(0, 0)];
        }
        const out = [];
        for (let i = 0; i < arr.length; i += chunk) {
            out.push(arr.slice(i, i + chunk));
        }
        return out;
    },
    sort: (list) => ensureArray(list).slice().sort(),
    sortDescending: (list) => ensureArray(list).slice().sort().reverse(),
    join: (list, delimiter = '', prefix, suffix) => {
        const arr = ensureArray(list);
        const body = arr.join(delimiter);
        if (prefix !== undefined && suffix !== undefined) {
            return `${prefix}${body}${suffix}`;
        }
        return body;
    },
    isEmpty: (list) => ensureArray(list).length === 0,
    find: (list, predicate) => {
        const arr = ensureArray(list);
        if (typeof predicate === 'function') {
            return arr.filter((item, idx) => predicate(item, idx));
        }
        return arr.filter((item) => item === predicate);
    },
    toString: (value) => `${value}`,
    length: (value) => (typeof value === 'string' ? [...value].length : ensureArray(value).length),
    toUpperCase: (value) => (typeof value === 'string' ? value.toUpperCase() : `${value}`.toUpperCase()),
    toLowerCase: (value) => (typeof value === 'string' ? value.toLowerCase() : `${value}`.toLowerCase()),
    trim: (value) => (typeof value === 'string' ? value.trim() : `${value}`.trim()),
    startsWith: (left, right) => `${left}`.startsWith(`${right}`),
    endsWith: (left, right) => `${left}`.endsWith(`${right}`),
    split: (left, right) => `${left}`.split(`${right}`),
    regexSplit: (left, right) => `${left}`.split(new RegExp(right, 'g')),
    substringBefore: (left, right) => {
        const str = `${left}`;
        const idx = str.indexOf(`${right}`);
        return idx === -1 ? '' : str.slice(0, idx);
    },
    substringAfter: (left, right) => {
        const str = `${left}`;
        const needle = `${right}`;
        const idx = str.indexOf(needle);
        return idx === -1 ? '' : str.slice(idx + needle.length);
    },
    charAt: (left, right) => `${left}`.charAt((Number(right) || 1) - 1),
    charCodeAt: (left, right) => `${left}`.charCodeAt((Number(right) || 1) - 1),
    lastIndexOf: (left, right) => `${left}`.lastIndexOf(`${right}`) + 1,
    repeat: (left, right) => `${left}`.repeat(Math.max(0, Number(right) || 0)),
    substring: (str, start, len) => {
        const s = `${str}`;
        const from = Math.max(0, (Number(start) || 1) - 1);
        if (len === undefined) return s.slice(from);
        return s.slice(from, from + Number(len));
    },
    replace: (str, pattern, replacement, flags) => `${str}`.replace(
        flags ? new RegExp(pattern, flags) : pattern,
        replacement
    ),
    regexReplace: (str, pattern, replacement, flags) => `${str}`.replace(
        new RegExp(pattern, flags || 'g'),
        replacement
    ),
    replaceFirst: (str, pattern, replacement) => `${str}`.replace(pattern, replacement),
    replaceLast: (str, pattern, replacement) => {
        const s = `${str}`;
        const idx = s.lastIndexOf(pattern);
        if (idx === -1) return s;
        return s.slice(0, idx) + replacement + s.slice(idx + pattern.length);
    },
    fromCharCode: (...codes) => String.fromCharCode(...codes.map((c) => Number(c) || 0)),
    padStart: (str, len, pad) => `${str}`.padStart(Number(len) || 0, `${pad}`),
    padEnd: (str, len, pad) => `${str}`.padEnd(Number(len) || 0, `${pad}`),
    sanitizeFilename: (str) => `${str}`.replace(/[\\\\/:*?"<>|]/g, '_').replace(/\\.+/g, ''),
    startswith: (left, right) => `${left}`.startsWith(`${right}`),
    endswith: (left, right) => `${left}`.endsWith(`${right}`),
    date: (value) => new Date(value),
    time: (value) => new Date(`1970-01-01T${value}`),
    datetime: (value) => new Date(value),
    duration: (value) => value,
    dayOfWeek: (value) => new Date(value).getDay() || 7,
    monthOfYear: (value) => new Date(value).getMonth() + 1,
    lastDayOfMonth: (value) => {
        const d = new Date(value);
        return new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
    },
};

export function installBuiltins(target = globalThis) {
    Object.entries(builtins).forEach(([name, fn]) => {
        target[name] = fn;
    });
    return target;
}
