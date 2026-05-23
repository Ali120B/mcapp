import { invoke } from '@tauri-apps/api/core';

export type OfflineAccount = {
  id: string;
  username: string;
  kind: 'offline';
};

export type AccountsState = {
  active_account_id: string | null;
  accounts: OfflineAccount[];
};

export type ModrinthProject = {
  project_id: string;
  title: string;
  description: string;
  icon_url: string | null;
  downloads: number;
  project_type: string;
  slug: string;
};

export async function getAccounts() {
  return invoke<AccountsState>('get_accounts');
}

export async function addOfflineAccount(username: string) {
  return invoke<AccountsState>('add_offline_account', { username });
}

export async function deleteAccount(accountId: string) {
  return invoke<AccountsState>('delete_account', { accountId });
}

export async function setActiveAccount(accountId: string) {
  return invoke<AccountsState>('set_active_account', { accountId });
}

export async function searchProjects(query: string, projectType = 'modpack', page = 0) {
  return invoke<ModrinthProject[]>('search_projects', { query, projectType, page });
}
