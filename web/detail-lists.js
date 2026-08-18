/** Client-side filter/page helpers for detail tab lists (pure JS). */

/**
 * @param {Array<{date?: string, question?: string, position?: string, result?: string}>} rows
 * @param {{ query?: string, position?: string, year?: string }} filters
 */
export function filterVotes(rows, filters = {}) {
  const list = Array.isArray(rows) ? rows : [];
  const q = String(filters.query || "")
    .trim()
    .toLowerCase();
  const pos = String(filters.position || "").trim();
  const year = String(filters.year || "").trim();
  return list.filter((v) => {
    if (pos && String(v.position || "") !== pos) return false;
    if (year) {
      const d = String(v.date || "");
      if (!d.startsWith(year)) return false;
    }
    if (q) {
      const hay = `${v.question || ""} ${v.position || ""} ${v.result || ""} ${v.date || ""}`.toLowerCase();
      if (!hay.includes(q)) return false;
    }
    return true;
  });
}

/**
 * @template T
 * @param {T[]} rows
 * @param {number} page 1-based
 * @param {number} pageSize
 * @returns {{ rows: T[], page: number, pageSize: number, total: number, totalPages: number }}
 */
export function pageSlice(rows, page, pageSize) {
  const list = Array.isArray(rows) ? rows : [];
  const size = Math.max(1, Math.min(100, Number(pageSize) || 10));
  const total = list.length;
  const totalPages = Math.max(1, Math.ceil(total / size) || 1);
  let p = Math.max(1, Number(page) || 1);
  if (p > totalPages) p = totalPages;
  const start = (p - 1) * size;
  return {
    rows: list.slice(start, start + size),
    page: p,
    pageSize: size,
    total,
    totalPages,
  };
}

/** Distinct years from vote dates (desc). */
export function voteYears(rows) {
  const set = new Set();
  for (const v of rows || []) {
    const d = String(v.date || "");
    const m = d.match(/^(\d{4})/);
    if (m) set.add(m[1]);
  }
  return [...set].sort((a, b) => b.localeCompare(a));
}

/** Distinct positions in data order of first appearance. */
export function votePositions(rows) {
  const seen = new Set();
  const out = [];
  for (const v of rows || []) {
    const p = String(v.position || "").trim();
    if (!p || seen.has(p)) continue;
    seen.add(p);
    out.push(p);
  }
  return out;
}

export function defaultVotesUi() {
  return { query: "", position: "", year: "", page: 1, pageSize: 10 };
}

/**
 * @param {Array<{name?: string, location?: string, date?: string, amount_display?: string}>} rows
 * @param {{ query?: string }} filters
 */
export function filterContributors(rows, filters = {}) {
  const list = Array.isArray(rows) ? rows : [];
  const q = String(filters.query || "")
    .trim()
    .toLowerCase();
  if (!q) return list;
  return list.filter((r) => {
    const hay = `${r.name || ""} ${r.location || ""} ${r.date || ""} ${r.amount_display || ""}`.toLowerCase();
    return hay.includes(q);
  });
}

/**
 * Tag contributor rows with kind for combined individuals/committees lists.
 * @param {object[]} individuals
 * @param {object[]} committees
 * @param {"all"|"individuals"|"committees"} kind
 */
export function mergeContributorLists(individuals, committees, kind = "all") {
  const ind = (Array.isArray(individuals) ? individuals : []).map((r) => ({
    ...r,
    _kind: "individuals",
  }));
  const cmte = (Array.isArray(committees) ? committees : []).map((r) => ({
    ...r,
    _kind: "committees",
  }));
  if (kind === "individuals") return ind;
  if (kind === "committees") return cmte;
  return ind.concat(cmte);
}

export function defaultFinanceUi() {
  return { query: "", kind: "all", page: 1, pageSize: 10 };
}
