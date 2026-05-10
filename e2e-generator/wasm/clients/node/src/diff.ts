export type Diff = {
  path: string;
  reason: string;
  left: unknown;
  right: unknown;
};

export function findDiff(left: unknown, right: unknown, path = '$'): Diff | null {
  if (Object.is(left, right)) {
    return null;
  }

  if (typeof left !== typeof right) {
    return { path, reason: 'type mismatch', left, right };
  }

  if (left === null || right === null) {
    return { path, reason: 'value mismatch', left, right };
  }

  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right)) {
      return { path, reason: 'array mismatch', left, right };
    }
    if (left.length !== right.length) {
      return { path, reason: 'array length mismatch', left: left.length, right: right.length };
    }
    for (let i = 0; i < left.length; i += 1) {
      const diff = findDiff(left[i], right[i], `${path}[${i}]`);
      if (diff) {
        return diff;
      }
    }
    return null;
  }

  if (typeof left === 'object' && typeof right === 'object') {
    const leftRecord = left as Record<string, unknown>;
    const rightRecord = right as Record<string, unknown>;
    const keys = Array.from(new Set([...Object.keys(leftRecord), ...Object.keys(rightRecord)])).sort();
    for (const key of keys) {
      if (!(key in leftRecord) || !(key in rightRecord)) {
        return { path: `${path}.${key}`, reason: 'missing key', left: leftRecord[key], right: rightRecord[key] };
      }
      const diff = findDiff(leftRecord[key], rightRecord[key], `${path}.${key}`);
      if (diff) {
        return diff;
      }
    }
    return null;
  }

  return { path, reason: 'value mismatch', left, right };
}
