import { NavLink, Route, Routes } from 'react-router-dom';
import { HomePage } from './pages/HomePage';
import { DiscoverPage } from './pages/DiscoverPage';
import { AccountsPage } from './pages/AccountsPage';

export function App() {
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <h1>MCApp</h1>
        <nav>
          <NavLink to="/">Home</NavLink>
          <NavLink to="/discover">Discover</NavLink>
          <NavLink to="/accounts">Accounts</NavLink>
        </nav>
      </aside>
      <main className="content">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/discover" element={<DiscoverPage />} />
          <Route path="/accounts" element={<AccountsPage />} />
        </Routes>
      </main>
    </div>
  );
}
