/** Keep model selection valid against the latest endpoint model list. */
export function reconcileModelSelection(
  modelNames: string[],
  current?: string,
): string | undefined {
  const trimmed = current?.trim();
  if (!trimmed) return undefined;
  return modelNames.includes(trimmed) ? trimmed : undefined;
}

/** Pick the best default when nothing is selected yet. */
export function pickDefaultModel(
  modelNames: string[],
  preferred?: string,
): string | undefined {
  const reconciled = reconcileModelSelection(modelNames, preferred);
  if (reconciled) return reconciled;
  return modelNames[0];
}