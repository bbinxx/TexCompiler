const COMPILE_ENDPOINT = '/compile';

export async function compileCode(code, compiler) {
  try {
    const response = await fetch(COMPILE_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, compiler }),
    });

    if (response.ok) {
      const blob = await response.blob();
      return { success: true, data: blob };
    }

    const errorData = await response.json();
    return {
      success: false,
      error: errorData.error || 'Compilation failed',
      log: errorData.log || null,
      warnings: errorData.warnings || [],
    };
  } catch (err) {
    return {
      success: false,
      error: 'Network error: unable to reach the server',
      log: err.message,
    };
  }
}
