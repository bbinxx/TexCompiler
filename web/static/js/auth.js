const KEY = 'texcompiler-api-token';
export function getApiToken() {
  try { return sessionStorage.getItem(KEY) || ''; } catch { return ''; }
}
export function setApiToken(token) {
  if (token) sessionStorage.setItem(KEY,token.trim());
  else sessionStorage.removeItem(KEY);
}
export function apiHeaders() {
  const token = getApiToken();
  return token ? {'Content-Type':'application/json',Authorization:`Bearer ${token}`} : {'Content-Type':'application/json'};
}
