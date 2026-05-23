import { useState } from 'react';
import { searchProjects, type ModrinthProject } from '../hooks/api';

export function DiscoverPage() {
  const [query, setQuery] = useState('adventure');
  const [items, setItems] = useState<ModrinthProject[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function runSearch() {
    setLoading(true);
    setError(null);
    try {
      const results = await searchProjects(query, 'modpack', 0);
      setItems(results);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section>
      <h2>Discover (Modrinth)</h2>
      <div className="toolbar">
        <input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search modpacks" />
        <button onClick={runSearch} disabled={loading}>{loading ? 'Searching…' : 'Search'}</button>
      </div>
      {error ? <p className="error">{error}</p> : null}
      <ul className="cards">
        {items.map((item) => (
          <li key={item.project_id}>
            <h3>{item.title}</h3>
            <p>{item.description}</p>
            <small>{item.downloads.toLocaleString()} downloads</small>
          </li>
        ))}
      </ul>
    </section>
  );
}
