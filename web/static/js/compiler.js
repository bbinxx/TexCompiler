import { apiHeaders } from './auth.js';
const COMPILE_ENDPOINT = '/compile';

export async function compileCode(code, compiler, project = null) {
  try {
    const response = await fetch(COMPILE_ENDPOINT, {
      method: 'POST',
      headers: apiHeaders(),
      body: JSON.stringify(project ? {...project, compiler} : { code, compiler }),
    });

    if (response.ok) {
      const blob = await response.blob();
      return { success: true, data: blob };
    }

    const errorData = await response.json();
    return {
      success: false,
      error: response.status === 401 ? 'Authentication required. Enter the API token in Settings.' : errorData.error || 'Compilation failed',
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
