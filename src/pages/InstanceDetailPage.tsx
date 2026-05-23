import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getInstanceState, importMrpack, installVersionToInstance, removeInstanceMod, setInstanceSettings, toggleInstanceMod, launchInstance, stopInstance, getRunningInstances, type InstanceSettings, type InstanceState } from '../hooks/api';
import { useTaskCenter } from '../components';

const empty: InstanceState = { mods: [], worlds: [], logs: [], settings: { memory_mb: 4096, width: 1280, height: 720 } };

export function InstanceDetailPage() {
  const { id = '' } = useParams();
  const [state, setState] = useState<InstanceState>(empty);
  const [projectId, setProjectId] = useState('');
  const [versionId, setVersionId] = useState('');
  const [mrpackPath, setMrpackPath] = useState('');
  const [running, setRunning] = useState(false);
  const [launchMsg, setLaunchMsg] = useState('');
  const { queueTask, updateTask } = useTaskCenter();

  const refresh = () => getInstanceState(id).then(setState);
  useEffect(() => { if (id) { void refresh(); void getRunningInstances().then(r=>setRunning(r.some(x=>x.instance_id===id))); } }, [id]);

  const saveSettings = (s: InstanceSettings) => setInstanceSettings(id, s).then(setState);

  return <section>
    <h2>Instance Detail · {id}</h2>
    <div className='toolbar'>
      <input placeholder='Project ID' value={projectId} onChange={e=>setProjectId(e.target.value)} />
      <input placeholder='Version ID' value={versionId} onChange={e=>setVersionId(e.target.value)} />
      <button onClick={()=>{ const taskId=`install-${id}`; queueTask({ id: taskId, instanceId: id, label: `Install content for ${id}`, kind: 'download', status: 'queued', step: 'queued', progress: 0 }); installVersionToInstance(id, projectId, versionId).then(setState).catch(err=>updateTask(taskId,{status:'error',step:'failed',error:String(err),progress:1})); }}>Install version</button>
    </div>
    <div className='toolbar'>
      <input placeholder='Path to .mrpack' value={mrpackPath} onChange={e=>setMrpackPath(e.target.value)} />
      <button onClick={()=>{ const taskId=`import-${id}`; queueTask({ id: taskId, instanceId: id, label: `Import modpack for ${id}`, kind: 'import', status: 'running', step: 'extracting', progress: 0.2 }); importMrpack(id, mrpackPath).then(st=>{ updateTask(taskId,{status:'done',step:'complete',progress:1}); setState(st); }).catch(err=>updateTask(taskId,{status:'error',step:'failed',error:String(err),progress:1})); }}>Import .mrpack</button>
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
      <label>RAM MB <input value={state.settings.memory_mb} type='number' onChange={e=>saveSettings({...state.settings,memory_mb:Number(e.target.value)})} /></label>
      <label>Width <input value={state.settings.width} type='number' onChange={e=>saveSettings({...state.settings,width:Number(e.target.value)})} /></label>
      <label>Height <input value={state.settings.height} type='number' onChange={e=>saveSettings({...state.settings,height:Number(e.target.value)})} /></label>
      <label>Java path <input value={state.settings.java_path ?? ''} onChange={e=>saveSettings({...state.settings,java_path:e.target.value || null})} /></label>
      <label>Pre-launch hook <input value={state.settings.pre_launch_hook ?? ''} onChange={e=>saveSettings({...state.settings,pre_launch_hook:e.target.value || null})} /></label>
      <label>Post-exit hook <input value={state.settings.post_exit_hook ?? ''} onChange={e=>saveSettings({...state.settings,post_exit_hook:e.target.value || null})} /></label>
    </div>

    <h3>Logs</h3>
    <pre style={{maxHeight:220,overflow:'auto',background:'#111',padding:12,borderRadius:8}}>{state.logs.join('\n') || 'No logs yet.'}</pre>
  </section>;
}
