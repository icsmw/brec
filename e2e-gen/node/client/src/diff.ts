export type Diff = {
  path: string;
  left: unknown;
  right: unknown;
  reason: string;
};

export function findDiff(leftValue: unknown, rightValue: unknown, path = '$'): Diff | null {
  if (Object.is(leftValue, rightValue)) {
    return null;
  }

  if (typeof leftValue !== typeof rightValue) {
    return {
      path,
      left: leftValue,
      right: rightValue,
      reason: 'type mismatch',
    };
  }

  if (leftValue === null || rightValue === null) {
    return {
      path,
      left: leftValue,
      right: rightValue,
      reason: 'null mismatch',
    };
  }

  if (Array.isArray(leftValue) || Array.isArray(rightValue)) {
    return findArrayDiff(leftValue, rightValue, path);
  }

  if (typeof leftValue === 'object' && typeof rightValue === 'object') {
    return findObjectDiff(leftValue, rightValue, path);
  }

  return {
    path,
    left: leftValue,
    right: rightValue,
    reason: 'value mismatch',
  };
}

function findArrayDiff(leftValue: unknown, rightValue: unknown, path: string): Diff | null {
  if (!Array.isArray(leftValue) || !Array.isArray(rightValue)) {
    return {
      path,
      left: leftValue,
      right: rightValue,
      reason: 'array mismatch',
    };
  }

  if (leftValue.length !== rightValue.length) {
    return {
      path: `${path}.length`,
      left: leftValue.length,
      right: rightValue.length,
      reason: 'array length mismatch',
    };
  }

  for (let i = 0; i < leftValue.length; i += 1) {
    const diff = findDiff(leftValue[i], rightValue[i], `${path}[${i}]`);
    if (diff) {
      return diff;
    }
  }

  return null;
}

function findObjectDiff(leftValue: object, rightValue: object, path: string): Diff | null {
  const left = leftValue as Record<string, unknown>;
  const right = rightValue as Record<string, unknown>;
  const keys = Array.from(new Set([...Object.keys(left), ...Object.keys(right)])).sort();

  for (const key of keys) {
    if (!(key in left) || !(key in right)) {
      return {
        path: `${path}.${key}`,
        left: left[key],
        right: right[key],
        reason: 'missing key',
      };
    }

    const diff = findDiff(left[key], right[key], `${path}.${key}`);
    if (diff) {
      return diff;
    }
  }

  return null;
}
