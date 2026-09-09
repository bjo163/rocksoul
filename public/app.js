const $ = (selector, root = document) => root.querySelector(selector);
const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];

const API = 'https://api.github.com';
const STATUS = new Set(['VERIFIED', 'OBSERVED', 'INFERRED', 'PENDING', 'BLOCKED', 'UNKNOWN']);

function parseEnv(text) {
  return Object.fromEntries(text.split(/\r?\n/).map(line => line.trim()).filter(line => line && !line.startsWith('#') && line.includes('=')).map(line => {
    const index = line.indexOf('=');
    return [line.slice(0, index), line.slice(index + 1)];
  }));
}

function repoUrl(repo) { return `https://github.com/${repo}`; }
function repoApi(repo) { return `${API}/repos/${repo}`; }

function resolveToken(value, brand) {
  if (!value || typeof value !== 'string' || !value.startsWith('@')) return value || null;
  const repos = {
    repo: brand.GITHUB_REPO,
    assets: brand.ASSETS_REPO,
    ui: brand.UI_REPO,
    mind: brand.MIND_REPO,
  };
  if (value === '@name World Core') return `${brand.NAME || 'RockSoul'} World Core`;
  if (value === '@repo') return repoUrl(repos.repo);
  if (value === '@repo-api') return repoApi(repos.repo);
  if (value === '@assets-repo') return repos.assets;
  if (value === '@ui-repo') return repos.ui;
  if (value === '@assets') return repoUrl(repos.assets);
  if (value === '@assets-api') return repoApi(repos.assets);
  if (value === '@ui') return repoUrl(repos.ui);
  if (value === '@ui-api') return repoApi(repos.ui);
  if (value === '@mind') return repoUrl(repos.mind);
  if (value === '@mind-api') return repoApi(repos.mind);
  if (value === '@project') return `https://github.com/users/${brand.GITHUB_OWNER}/projects?query=${encodeURIComponent(brand.PROJECT_NAME || brand.SLUG)}`;
  if (value === '@wiki') return `${repoUrl(repos.repo)}/wiki`;
  if (value === '@pages') return brand.PAGES_URL || null;
  if (value === '@vercel') return brand.VERCEL_URL || null;
  return null;
}

function resolvedWorld(world, brand) {
  return {
    ...world,
    assets: { ...world.assets, repository: resolveToken(world.assets.repository, brand) },
    ui: { ...world.ui, repository: resolveToken(world.ui.repository, brand) },
    places: world.places.map(place => ({
      ...place,
      name: resolveToken(place.name, brand) || place.name,
      href: resolveToken(place.href, brand),
      api: resolveToken(place.api, brand),
    })),
  };
}

async function json(url) {
  const response = await fetch(url, { headers: { Accept: 'application/vnd.github+json' } });
  if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
  return response.json();
}

function text(tag, value, className) {
  const element = document.createElement(tag);
  if (className) element.className = className;
  element.textContent = value ?? '';
  return element;
}

function evidence(value) {
  const normalized = STATUS.has(value) ? value : 'UNKNOWN';
  return text('span', normalized, `evidence ${normalized.toLowerCase()}`);
}

function activate(screenName) {
  $$('.screen').forEach(screen => screen.classList.toggle('active', screen.dataset.screen === screenName));
  $$('.nav button').forEach(button => button.classList.toggle('active', button.dataset.target === screenName));
  history.replaceState(null, '', `#${screenName.toLowerCase()}`);
}

function currentScreen() {
  const fromHash = location.hash.replace('#', '').toUpperCase();
  const valid = ['WORLD', 'MAP', 'QUESTS', 'CODEX', 'SYSTEM'];
  if (valid.includes(fromHash)) return fromHash;
  return location.hostname.endsWith('github.io') ? 'CODEX' : 'WORLD';
}

function renderIdentity(brand) {
  const name = brand.NAME || 'RockSoul';
  document.title = `${name} — ${location.hostname.endsWith('github.io') ? 'Codex' : 'World'}`;
  $('#brand-name').textContent = name;
  $('#hero-name').textContent = name;
  $('#hero-copy').textContent = brand.TAGLINE || 'Local-first digital cognitive runtime';
  $('#phase').textContent = brand.PHASE || 'Phase A / Birth';
  $('#footer-identity').textContent = `${name} • GEN ${brand.GENERATION || 1} • AGE ${brand.COGNITIVE_AGE || 0} • LV ${brand.LEVEL || 1}`;
  const metrics = [
    ['COGNITIVE AGE', brand.COGNITIVE_AGE || '0', 'Advances only through evaluation gates'],
    ['LEVEL', brand.LEVEL || '1', 'Capability progression, not decoration'],
    ['XP', brand.XP || '0', 'Only validated outcomes earn XP'],
    ['TRUST', brand.TRUST || '0', 'Never bypasses authorization policy'],
  ];
  $('#life-metrics').replaceChildren(...metrics.map(([label, value, sub]) => {
    const card = document.createElement('div');
    card.className = 'metric';
    card.append(text('div', label, 'metric-label'), text('div', value, 'metric-value'), text('div', sub, 'metric-sub'));
    return card;
  }));
}

function renderWorldMap(world) {
  const map = $('#map-grid');
  map.replaceChildren();
  const order = ['assets', 'mind', 'ui', 'world-core', 'project', 'wiki', 'pages', 'vercel', 'cloudflare'];
  const places = new Map(world.places.map(place => [place.id, place]));
  for (const id of order) {
    const place = places.get(id);
    if (!place) continue;
    const node = document.createElement(place.href ? 'a' : 'div');
    node.className = `node${id === 'world-core' ? ' core' : ''}`;
    if (place.href) { node.href = place.href; node.target = '_blank'; node.rel = 'noreferrer'; }
    node.append(text('div', place.kind, 'kind'), text('strong', place.name));
    const bottom = document.createElement('div');
    bottom.append(evidence(place.evidence));
    if (place.note) bottom.append(text('div', place.note, 'meta'));
    node.append(bottom);
    map.append(node);
  }
  $('#fog').replaceChildren(...world.discovery.map((state, index) => text('span', state, `chip${index <= 2 ? ' active' : ''}`)));
  $('#asset-provenance').textContent = `ASSETS accepted ${world.assets.acceptedRevision.slice(0, 12)} • UI ${world.ui.revision.slice(0, 12)} • ${world.ui.package}@${world.ui.version}`;
}

function renderQuests(issues) {
  const quests = issues.filter(issue => !issue.pull_request).slice(0, 12);
  if (!quests.length) { $('#quest-list').replaceChildren(text('div', 'No open quests observed.', 'empty')); return; }
  $('#quest-list').replaceChildren(...quests.map(issue => {
    const item = document.createElement('article'); item.className = 'quest';
    const head = document.createElement('div'); head.className = 'quest-head';
    const link = text('a', `#${issue.number} ${issue.title}`, 'quest-title'); link.href = issue.html_url; link.target = '_blank'; link.rel = 'noreferrer';
    head.append(link, evidence('OBSERVED')); item.append(head);
    item.append(text('div', `CHAPTER ${issue.milestone?.title || 'No chapter'} • updated ${new Date(issue.updated_at).toLocaleString()}`, 'meta'));
    const labels = document.createElement('div'); labels.className = 'labels';
    for (const label of issue.labels.slice(0, 7)) labels.append(text('span', label.name, 'label'));
    item.append(labels); return item;
  }));
}

function renderEvents(events, repoName) {
  if (!events.length) { $('#event-list').replaceChildren(text('div', 'GitHub evidence stream unavailable.', 'empty')); return; }
  $('#event-list').replaceChildren(...events.slice(0, 8).map(event => {
    const item = document.createElement('div'); item.className = 'event';
    item.append(text('time', new Date(event.created_at).toLocaleString()));
    item.append(text('p', `${event.type} observed on ${event.repo?.name || repoName}. This is GitHub evidence, not runtime telemetry.`));
    return item;
  }));
}

function renderSystems(world, observedRepos, brand) {
  const repoByName = new Map(observedRepos.filter(Boolean).map(repo => [repo.full_name, repo]));
  const rows = [
    ['WORLD CORE', brand.GITHUB_REPO, repoByName.has(brand.GITHUB_REPO) ? 'OBSERVED' : 'UNKNOWN'],
    ['WORLD RESOURCE', brand.ASSETS_REPO, repoByName.has(brand.ASSETS_REPO) ? 'OBSERVED' : 'UNKNOWN'],
    ['UI GRAMMAR', brand.UI_REPO, repoByName.has(brand.UI_REPO) ? 'OBSERVED' : 'UNKNOWN'],
    ['COGNITION LAB', brand.MIND_REPO, repoByName.has(brand.MIND_REPO) ? 'OBSERVED' : 'UNKNOWN'],
    ['SELF-HOSTED NODES', 'Runner inventory requires authenticated Actions runner API', 'UNKNOWN'],
    ['VERCEL PORTAL', brand.VERCEL_URL || `${brand.VERCEL_PROJECT || brand.SLUG} — canonical deployment not verified`, 'PENDING'],
    ['CLOUDFLARE GATEWAY', 'Private-first; no justified tunnel target yet', 'PENDING'],
  ];
  $('#system-list').replaceChildren(...rows.map(([kind, value, state]) => {
    const item = document.createElement('div'); item.className = 'system-item';
    const head = document.createElement('div'); head.className = 'system-head'; head.append(text('strong', kind), evidence(state));
    item.append(head, text('div', value, 'meta')); return item;
  }));
  $('#system-provenance').textContent = `Topology contract world.json schema v${world.schemaVersion}; statuses never upgrade beyond available evidence.`;
}

function renderCodex(brand) {
  const base = repoUrl(brand.GITHUB_REPO);
  const docs = name => `${base}/blob/main/docs/${name}.md`;
  const entries = [
    ['WORLD ENTRY', 'README', `${base}#readme`],
    ['WORLD', 'Digital World and evidence semantics', docs('WORLD')],
    ['ARCHITECTURE', 'System boundary and cognitive model', docs('ARCHITECTURE')],
    ['ROADMAP', 'Birth → World → Memory → cognition research', docs('ROADMAP')],
    ['STORAGE', 'Local-first placement and cloud boundaries', docs('STORAGE')],
    ['AUTOMATION', 'Gate + sync responsibility matrix', docs('AUTOMATION')],
    ['PROJECT', 'Issues, chapters, labels and World Map', docs('PROJECT_MANAGEMENT')],
    ['ARCHIVE', 'GitHub Wiki mirror', `${base}/wiki`],
  ];
  $('#codex-list').replaceChildren(...entries.map(([kind, title, href]) => {
    const item = document.createElement('a'); item.className = 'codex-item'; item.href = href; item.target = '_blank'; item.rel = 'noreferrer';
    item.append(text('div', kind, 'kind'), text('strong', title), text('div', href, 'meta')); return item;
  }));
}

function configureLinks(brand) {
  const base = repoUrl(brand.GITHUB_REPO);
  const links = {
    'core-link': base,
    'release-link': `${base}/releases`,
    'quest-github': `${base}/issues`,
  };
  for (const [id, href] of Object.entries(links)) { const element = $(`#${id}`); if (element) element.href = href; }
}

async function boot() {
  $$('.nav button').forEach(button => button.addEventListener('click', () => activate(button.dataset.target)));
  $$('[data-jump]').forEach(link => link.addEventListener('click', event => { event.preventDefault(); activate(link.dataset.jump); }));
  document.addEventListener('keydown', event => { const screens = ['WORLD', 'MAP', 'QUESTS', 'CODEX', 'SYSTEM']; const index = Number(event.key) - 1; if (index >= 0 && index < screens.length) activate(screens[index]); });
  activate(currentScreen());

  const [brandText, rawWorld] = await Promise.all([
    fetch('brand.env', { cache: 'no-store' }).then(response => { if (!response.ok) throw new Error('brand.env unavailable'); return response.text(); }),
    fetch('world.json', { cache: 'no-store' }).then(response => { if (!response.ok) throw new Error('world.json unavailable'); return response.json(); }),
  ]);
  const brand = parseEnv(brandText);
  const world = resolvedWorld(rawWorld, brand);
  renderIdentity(brand); renderWorldMap(world); renderCodex(brand); configureLinks(brand);

  const repo = brand.GITHUB_REPO;
  const requests = await Promise.allSettled([
    json(repoApi(repo)), json(`${repoApi(repo)}/issues?state=open&per_page=30`), json(`${repoApi(repo)}/events?per_page=12`), json(`${repoApi(repo)}/releases/latest`),
    json(repoApi(brand.ASSETS_REPO)), json(repoApi(brand.UI_REPO)), json(repoApi(brand.MIND_REPO)),
  ]);
  const value = index => requests[index].status === 'fulfilled' ? requests[index].value : null;
  const repository = value(0), issues = value(1) || [], events = value(2) || [], release = value(3);
  renderQuests(issues); renderEvents(events, repo); renderSystems(world, [repository, value(4), value(5), value(6)], brand);
  $('#world-status').replaceChildren(evidence(repository ? 'OBSERVED' : 'UNKNOWN'));
  $('#repo-branch').textContent = repository?.default_branch || 'unknown';
  $('#open-quests').textContent = String(issues.filter(issue => !issue.pull_request).length || 0);
  $('#release-version').textContent = release?.tag_name || 'none';
  $('#repo-updated').textContent = repository?.pushed_at ? new Date(repository.pushed_at).toLocaleString() : 'unknown';
  $('#observed-at').textContent = `Observed ${new Date().toLocaleString()} via public GitHub API`;
  $('#presence-dot').classList.toggle('ok', Boolean(repository));
  $('#presence-text').textContent = repository ? 'WORLD OBSERVED' : 'WORLD UNKNOWN';
}

boot().catch(error => {
  $('#presence-dot').classList.add('bad'); $('#presence-text').textContent = 'WORLD DEGRADED';
  $('#world-status').replaceChildren(evidence('UNKNOWN'));
  $('#event-list').replaceChildren(text('div', `Portal bootstrap failed: ${error.message}`, 'empty'));
});
