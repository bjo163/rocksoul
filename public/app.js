const $ = (selector, root = document) => root.querySelector(selector);
const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];

const API = 'https://api.github.com';
const REPO = 'bjo163/rocksoul';
const STATUS = new Set(['VERIFIED', 'OBSERVED', 'INFERRED', 'PENDING', 'BLOCKED', 'UNKNOWN']);

function parseEnv(text) {
  return Object.fromEntries(
    text.split(/\r?\n/)
      .map(line => line.trim())
      .filter(line => line && !line.startsWith('#') && line.includes('='))
      .map(line => {
        const index = line.indexOf('=');
        return [line.slice(0, index), line.slice(index + 1)];
      }),
  );
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
  const span = text('span', normalized, `evidence ${normalized.toLowerCase()}`);
  return span;
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
  document.title = `${brand.NAME || 'RockSoul'} — ${location.hostname.endsWith('github.io') ? 'Codex' : 'World'}`;
  $('#brand-name').textContent = brand.NAME || 'RockSoul';
  $('#hero-name').textContent = brand.NAME || 'RockSoul';
  $('#hero-copy').textContent = brand.TAGLINE || 'Local-first digital cognitive runtime';
  $('#phase').textContent = brand.PHASE || 'Phase A / Birth';
  $('#footer-identity').textContent = `${brand.NAME || 'RockSoul'} • GEN ${brand.GENERATION || 1} • AGE ${brand.COGNITIVE_AGE || 0} • LV ${brand.LEVEL || 1}`;
  const metrics = [
    ['COGNITIVE AGE', brand.COGNITIVE_AGE || '0', 'Advances only through evaluation gates'],
    ['LEVEL', brand.LEVEL || '1', 'Capability progression, not decoration'],
    ['XP', brand.XP || '0', 'Only validated outcomes earn XP'],
    ['TRUST', brand.TRUST || '0', 'Never bypasses authorization policy'],
  ];
  const container = $('#life-metrics');
  container.replaceChildren(...metrics.map(([label, value, sub]) => {
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
    if (place.href) {
      node.href = place.href;
      node.target = '_blank';
      node.rel = 'noreferrer';
    }
    node.append(text('div', place.kind, 'kind'));
    node.append(text('strong', place.name));
    const bottom = document.createElement('div');
    bottom.append(evidence(place.evidence));
    if (place.note) bottom.append(text('div', place.note, 'meta'));
    node.append(bottom);
    map.append(node);
  }
  const fog = $('#fog');
  fog.replaceChildren(...world.discovery.map((state, index) => {
    const chip = text('span', state, `chip${index <= 2 ? ' active' : ''}`);
    return chip;
  }));
  $('#asset-provenance').textContent = `ASSETS accepted ${world.assets.acceptedRevision.slice(0, 12)} • UI ${world.ui.revision.slice(0, 12)} • ${world.ui.package}@${world.ui.version}`;
}

function renderQuests(issues) {
  const list = $('#quest-list');
  const quests = issues.filter(issue => !issue.pull_request).slice(0, 12);
  if (!quests.length) {
    list.replaceChildren(text('div', 'No open quests observed.', 'empty'));
    return;
  }
  list.replaceChildren(...quests.map(issue => {
    const item = document.createElement('article');
    item.className = 'quest';
    const head = document.createElement('div');
    head.className = 'quest-head';
    const link = text('a', `#${issue.number} ${issue.title}`, 'quest-title');
    link.href = issue.html_url;
    link.target = '_blank';
    link.rel = 'noreferrer';
    head.append(link, evidence('OBSERVED'));
    item.append(head);
    const milestone = issue.milestone?.title || 'No chapter';
    item.append(text('div', `CHAPTER ${milestone} • updated ${new Date(issue.updated_at).toLocaleString()}`, 'meta'));
    const labels = document.createElement('div');
    labels.className = 'labels';
    for (const label of issue.labels.slice(0, 7)) labels.append(text('span', label.name, 'label'));
    item.append(labels);
    return item;
  }));
}

function renderEvents(events) {
  const list = $('#event-list');
  if (!events.length) {
    list.replaceChildren(text('div', 'GitHub evidence stream unavailable.', 'empty'));
    return;
  }
  list.replaceChildren(...events.slice(0, 8).map(event => {
    const item = document.createElement('div');
    item.className = 'event';
    item.append(text('time', new Date(event.created_at).toLocaleString()));
    item.append(text('p', `${event.type} observed on ${event.repo?.name || REPO}. This is GitHub evidence, not runtime telemetry.`));
    return item;
  }));
}

function renderSystems(world, observedRepos) {
  const list = $('#system-list');
  const repoByName = new Map(observedRepos.filter(Boolean).map(repo => [repo.full_name, repo]));
  const rows = [
    ['WORLD CORE', 'bjo163/rocksoul', repoByName.has('bjo163/rocksoul') ? 'OBSERVED' : 'UNKNOWN'],
    ['WORLD RESOURCE', 'bjo163/rocksoul-assets', repoByName.has('bjo163/rocksoul-assets') ? 'OBSERVED' : 'UNKNOWN'],
    ['UI GRAMMAR', 'bjo163/rocksoul-ui', repoByName.has('bjo163/rocksoul-ui') ? 'OBSERVED' : 'UNKNOWN'],
    ['COGNITION LAB', 'bjo163/rocksoul-mind', repoByName.has('bjo163/rocksoul-mind') ? 'OBSERVED' : 'UNKNOWN'],
    ['SELF-HOSTED NODES', 'Runner inventory requires authenticated Actions runner API', 'UNKNOWN'],
    ['VERCEL PORTAL', 'Canonical bjo163/rocksoul project not verified', 'PENDING'],
    ['CLOUDFLARE GATEWAY', 'Private-first; no justified tunnel target yet', 'PENDING'],
  ];
  list.replaceChildren(...rows.map(([kind, value, state]) => {
    const item = document.createElement('div');
    item.className = 'system-item';
    const head = document.createElement('div');
    head.className = 'system-head';
    head.append(text('strong', kind), evidence(state));
    item.append(head, text('div', value, 'meta'));
    return item;
  }));
  $('#system-provenance').textContent = `Topology contract world.json schema v${world.schemaVersion}; statuses never upgrade beyond available evidence.`;
}

function renderCodex() {
  const entries = [
    ['WORLD ENTRY', 'README', 'https://github.com/bjo163/rocksoul#readme'],
    ['ARCHITECTURE', 'System boundary and cognitive model', 'https://github.com/bjo163/rocksoul/blob/feature/repo-platform/docs/ARCHITECTURE.md'],
    ['ROADMAP', 'Birth → World → Memory → cognition research', 'https://github.com/bjo163/rocksoul/blob/feature/repo-platform/docs/ROADMAP.md'],
    ['STORAGE', 'Local-first placement and cloud boundaries', 'https://github.com/bjo163/rocksoul/blob/feature/repo-platform/docs/STORAGE.md'],
    ['AUTOMATION', 'Gate + sync responsibility matrix', 'https://github.com/bjo163/rocksoul/blob/feature/repo-platform/docs/AUTOMATION.md'],
    ['PROJECT', 'Issues, chapters, labels and World Map', 'https://github.com/bjo163/rocksoul/blob/feature/repo-platform/docs/PROJECT_MANAGEMENT.md'],
    ['ARCHIVE', 'GitHub Wiki mirror (activation pending)', 'https://github.com/bjo163/rocksoul/wiki'],
  ];
  const list = $('#codex-list');
  list.replaceChildren(...entries.map(([kind, title, href]) => {
    const item = document.createElement('a');
    item.className = 'codex-item';
    item.href = href;
    item.target = '_blank';
    item.rel = 'noreferrer';
    item.append(text('div', kind, 'kind'), text('strong', title), text('div', href, 'meta'));
    return item;
  }));
}

async function boot() {
  $$('.nav button').forEach(button => button.addEventListener('click', () => activate(button.dataset.target)));
  document.addEventListener('keydown', event => {
    const screens = ['WORLD', 'MAP', 'QUESTS', 'CODEX', 'SYSTEM'];
    const index = Number(event.key) - 1;
    if (index >= 0 && index < screens.length) activate(screens[index]);
  });
  activate(currentScreen());
  renderCodex();

  const [brandText, world] = await Promise.all([
    fetch('brand.env', { cache: 'no-store' }).then(response => {
      if (!response.ok) throw new Error('brand.env unavailable');
      return response.text();
    }),
    fetch('world.json', { cache: 'no-store' }).then(response => {
      if (!response.ok) throw new Error('world.json unavailable');
      return response.json();
    }),
  ]);

  renderIdentity(parseEnv(brandText));
  renderWorldMap(world);

  const requests = await Promise.allSettled([
    json(`${API}/repos/${REPO}`),
    json(`${API}/repos/${REPO}/issues?state=open&per_page=30`),
    json(`${API}/repos/${REPO}/events?per_page=12`),
    json(`${API}/repos/${REPO}/releases/latest`),
    json(`${API}/repos/bjo163/rocksoul-assets`),
    json(`${API}/repos/bjo163/rocksoul-ui`),
    json(`${API}/repos/bjo163/rocksoul-mind`),
  ]);

  const value = index => requests[index].status === 'fulfilled' ? requests[index].value : null;
  const repo = value(0);
  const issues = value(1) || [];
  const events = value(2) || [];
  const release = value(3);
  const observedRepos = [repo, value(4), value(5), value(6)];

  renderQuests(issues);
  renderEvents(events);
  renderSystems(world, observedRepos);

  $('#world-status').replaceChildren(evidence(repo ? 'OBSERVED' : 'UNKNOWN'));
  $('#repo-branch').textContent = repo?.default_branch || 'unknown';
  $('#open-quests').textContent = String(issues.filter(issue => !issue.pull_request).length || 0);
  $('#release-version').textContent = release?.tag_name || 'none';
  $('#repo-updated').textContent = repo?.pushed_at ? new Date(repo.pushed_at).toLocaleString() : 'unknown';
  $('#observed-at').textContent = `Observed ${new Date().toLocaleString()} via public GitHub API`;
  $('#presence-dot').classList.toggle('ok', Boolean(repo));
  $('#presence-text').textContent = repo ? 'WORLD OBSERVED' : 'WORLD UNKNOWN';
}

boot().catch(error => {
  $('#presence-dot').classList.add('bad');
  $('#presence-text').textContent = 'WORLD DEGRADED';
  $('#world-status').replaceChildren(evidence('UNKNOWN'));
  $('#event-list').replaceChildren(text('div', `Portal bootstrap failed: ${error.message}`, 'empty'));
});
