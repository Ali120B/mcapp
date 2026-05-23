import { useEffect, useState } from 'react';
import { addOfflineAccount, deleteAccount, getAccounts, setActiveAccount, type AccountsState } from '../hooks/api';

export function AccountsPage() {
  const [state, setState] = useState<AccountsState>({ active_account_id: null, accounts: [] });
  const [name, setName] = useState('');

  async function refresh() {
    setState(await getAccounts());
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function add() {
    if (!name.trim()) return;
    setState(await addOfflineAccount(name.trim()));
    setName('');
  }

  return (
    <section>
      <h2>Offline Accounts</h2>
      <div className="toolbar">
        <input placeholder="Player name" value={name} onChange={(e) => setName(e.target.value)} />
        <button onClick={add}>Add offline account</button>
      </div>
      <ul className="cards">
        {state.accounts.map((account) => (
          <li key={account.id}>
            <h3>{account.username}</h3>
            <small>{account.id}</small>
            <div className="row">
              <button onClick={() => setActiveAccount(account.id).then(setState)}>
                {state.active_account_id === account.id ? 'Active' : 'Set active'}
              </button>
              <button onClick={() => deleteAccount(account.id).then(setState)}>Delete</button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
