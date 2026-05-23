import { invoke } from '@tauri-apps/api/core';
export type OfflineAccount={id:string;username:string;kind:'offline'};
export type AccountsState={active_account_id:string|null;accounts:OfflineAccount[]};
export type ModrinthProject={id?:string;project_id?:string;slug?:string;title:string;description:string;body?:string;icon_url:string|null;downloads?:number;followers?:number;project_type:string;author?:string;categories?:string[]};
export type ModrinthVersion={id:string;name:string;version_number:string;game_versions:string[];loaders:string[];date_published:string};
export type SearchResponse={hits:ModrinthProject[];total_hits?:number};
export type InstanceSummary={id:string;name:string;mc_version:string;loader:string;last_played?:string|null;group?:string|null};
export type InstancesState={instances:InstanceSummary[]};
export type InstanceModEntry={file_name:string;project_id:string;version_id:string;enabled:boolean;path:string};
export type InstanceSettings={memory_mb:number;java_path?:string|null;width:number;height:number;pre_launch_hook?:string|null;post_exit_hook?:string|null};
export type InstanceState={mods:InstanceModEntry[];worlds:string[];logs:string[];settings:InstanceSettings};

export const getAccounts=()=>invoke<AccountsState>('get_accounts');
export const addOfflineAccount=(username:string)=>invoke<AccountsState>('add_offline_account',{username});
export const deleteAccount=(accountId:string)=>invoke<AccountsState>('delete_account',{accountId});
export const setActiveAccount=(accountId:string)=>invoke<AccountsState>('set_active_account',{accountId});
export const searchProjects=(args:any)=>invoke<SearchResponse>('search_projects',args);
export const getProject=(idOrSlug:string)=>invoke<ModrinthProject>('get_project',{idOrSlug});
export const getProjectVersions=(projectId:string)=>invoke<ModrinthVersion[]>('get_project_versions',{projectId});
export const listInstances=()=>invoke<InstancesState>('list_instances');
export const createInstance=(name:string,mcVersion:string,loader:string)=>invoke<InstancesState>('create_instance',{name,mcVersion,loader});
export const deleteInstance=(instanceId:string)=>invoke<InstancesState>('delete_instance',{instanceId});
export const getInstanceState=(instanceId:string)=>invoke<InstanceState>('get_instance_state',{instanceId});
export const setInstanceSettings=(instanceId:string,settings:InstanceSettings)=>invoke<InstanceState>('set_instance_settings',{instanceId,settings});
export const toggleInstanceMod=(instanceId:string,fileName:string,enabled:boolean)=>invoke<InstanceState>('toggle_instance_mod',{instanceId,fileName,enabled});
export const removeInstanceMod=(instanceId:string,fileName:string)=>invoke<InstanceState>('remove_instance_mod',{instanceId,fileName});
export const installVersionToInstance=(instanceId:string,projectId:string,versionId:string)=>invoke<InstanceState>('install_version_to_instance',{instanceId,projectId,versionId});
export const importMrpack=(instanceId:string,mrpackPath:string)=>invoke<InstanceState>('import_mrpack',{instanceId,mrpackPath});

export type JavaInstallation={path:string;version:string;major:number};
export type JavaRecommendation={mc_version:string;required_major:number;installed_match:JavaInstallation|null};
export const detectJavaInstallations=()=>invoke<JavaInstallation[]>('detect_java_installations');
export const recommendJavaForMc=(mcVersion:string)=>invoke<JavaRecommendation>('recommend_java_for_mc',{mcVersion});
export const downloadAdoptiumJava=(major:number)=>invoke<string>('download_adoptium_java',{major});

export type RunningInstance={instance_id:string;pid:number;started_at:string};
export const launchInstance=(instanceId:string)=>invoke<string>('launch_instance',{instanceId});
export const stopInstance=(instanceId:string)=>invoke<boolean>('stop_instance',{instanceId});
export const getRunningInstances=()=>invoke<RunningInstance[]>('get_running_instances');

export const setInstanceGroup=(instanceId:string,group:string|null)=>invoke<InstancesState>('set_instance_group',{instanceId,group});
export const duplicateInstance=(instanceId:string,newName:string)=>invoke<InstancesState>('duplicate_instance',{instanceId,newName});
