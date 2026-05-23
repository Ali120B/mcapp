import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getInstanceState, importMrpack, installVersionToInstance, removeInstanceMod, setInstanceSettings, toggleInstanceMod, launchInstance, stopInstance, getRunningInstances, type InstanceSettings, type InstanceState } from '../hooks/api';

const empty: InstanceState = { mods: [], worlds: [], logs: [], settings: { memory_mb: 4096, width: 1280, height: 720 } };

export function InstanceDetailPage() {
  const { id = '' } = useParams();
  const [state, setState] = useState<InstanceState>(empty);
  const [projectId, setProjectId] = useState('');
  const [versionId, setVersionId] = useState('');
  const [mrpackPath, setMrpackPath] = useState('');
  const [running, setRunning] = useState(false);
  const [launchMsg, setLaunchMsg] = useState('');

  const refresh = () => getInstanceState(id).then(setState);
  useEffect(() => { if (id) { void refresh(); void getRunningInstances().then(r=>setRunning(r.some(x=>x.instance_id===id))); } }, [id]);

  const saveSettings = (s: InstanceSettings) => setInstanceSettings(id, s).then(setState);

  return <section>
    <h2>Instance Detail · {id}</h2>
    <div className='toolbar'>
      <input placeholder='Project ID' value={projectId} onChange={e=>setProjectId(e.target.value)} />
      <input placeholder='Version ID' value={versionId} onChange={e=>setVersionId(e.target.value)} />
      <button onClick={()=>installVersionToInstance(id, projectId, versionId).then(setState)}>Install version</button>
    </div>
    <div className='toolbar'>
      <input placeholder='Path to .mrpack' value={mrpackPath} onChange={e=>setMrpackPath(e.target.value)} />
      <button onClick={()=>importMrpack(id, mrpackPath).then(setState)}>Import .mrpack</button>
    </div>

    <h3>Launch</h3>
    <div className='toolbar'>
      {!running ? <button onClick={()=>launchInstance(id).then(m=>{setLaunchMsg(m); setRunning(true);})}>Play instance</button> : <button onClick={()=>stopInstance(id).then(()=>{setLaunchMsg('Stopped'); setRunning(false);})}>Stop instance</button>}
      <span>{running ? 'Running' : 'Not running'} {launchMsg}</span>
    </div>

    <h3>Content</h3>
    <ul className='cards'>{state.mods.map(m=><li key={m.file_name}><b>{m.file_name}</b><p>{m.project_id} · {m.version_id}</p><button onClick={()=>toggleInstanceMod(id,m.file_name,!m.enabled).then(setState)}>{m.enabled?'Disable':'Enable'}</button><button onClick={()=>removeInstanceMod(id,m.file_name).then(setState)}>Remove</button></li>)}</ul>

    <h3>Worlds</h3>
    <ul className='cards'>{state.worlds.map(w=><li key={w}>{w}</li>)}{!state.worlds.length && <li>No worlds found yet.</li>}</ul>

    <h3>Settings</h3>
    <div className='toolbar'>
      <input value={state.settings.memory_mb} type='number' onChange={e=>saveSettings({...state.settings,memory_mb:Number(e.target.value)})} />
      <input value={state.settings.width} type='number' onChange={e=>saveSettings({...state.settings,width:Number(e.target.value)})} />
      <input value={state.settings.height} type='number' onChange={e=>saveSettings({...state.settings,height:Number(e.target.value)})} />
    </div>
  </section>;
}
