import React, { createContext, useContext, useEffect, useMemo, useState, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import { listen } from '@tauri-apps/api/event';

type Toast = { id: number; kind: 'success'|'error'|'info'; message: string };

type TaskKind = 'download' | 'import';
type TaskStatus = 'queued' | 'running' | 'done' | 'error';
type TaskItem = { id: string; instanceId: string; label: string; kind: TaskKind; status: TaskStatus; step: string; progress: number; updatedAt: number; error?: string };

type TaskCenterCtxValue = {
  queueTask: (task: Omit<TaskItem, 'updatedAt'>) => void;
  updateTask: (id: string, patch: Partial<TaskItem>) => void;
};
const TASKS_KEY = 'pulsar-task-center-v1';
const TaskCenterCtx = createContext<TaskCenterCtxValue>({ queueTask: () => {}, updateTask: () => {} });
export function useTaskCenter(){ return useContext(TaskCenterCtx); }

const ToastCtx = createContext<{push:(kind:Toast['kind'],message:string)=>void}>({push:()=>{}});
export function useToast(){ return useContext(ToastCtx); }

export function AppProviders({children}:{children:ReactNode}){
  const [toasts,setToasts]=useState<Toast[]>([]);
  const [tasks,setTasks]=useState<Record<string,TaskItem>>(()=>{
    try { return JSON.parse(localStorage.getItem(TASKS_KEY) ?? '{}') as Record<string, TaskItem>; } catch { return {}; }
  });
  const push=(kind:Toast['kind'],message:string)=>{ const id=Date.now()+Math.random(); setToasts(t=>[...t,{id,kind,message}]); setTimeout(()=>setToasts(t=>t.filter(x=>x.id!==id)),3000); };
  const queueTask: TaskCenterCtxValue['queueTask'] = (task)=>setTasks(prev=>({ ...prev, [task.id]: { ...task, updatedAt: Date.now() } }));
  const updateTask: TaskCenterCtxValue['updateTask'] = (id, patch)=>setTasks(prev=>({ ...prev, [id]: { ...prev[id], ...patch, updatedAt: Date.now() } }));

  useEffect(()=>{ localStorage.setItem(TASKS_KEY, JSON.stringify(tasks)); },[tasks]);

  useEffect(()=>{ let unlisten: undefined | (()=>void); void listen<any>('install:progress', (event)=>{
    const p = event.payload as { instance_id:string; step:string; progress:number };
    const id = `install-${p.instance_id}`;
    queueTask({ id, instanceId: p.instance_id, label: `Install content for ${p.instance_id}`, kind: 'download', status: p.progress >= 1 ? 'done' : 'running', step: p.step, progress: p.progress });
    if (p.progress >= 1) push('success', `Install complete for ${p.instance_id}`);
  }).then(fn=>{ unlisten=fn; }); return ()=>{ if(unlisten) unlisten();}; },[]);
  const orderedTasks = useMemo(()=>Object.values(tasks).sort((a,b)=>b.updatedAt-a.updatedAt),[tasks]);

  return <ToastCtx.Provider value={{push}}><TaskCenterCtx.Provider value={{queueTask, updateTask}}>{children}
    <div className='toast-stack'>{toasts.map(t=><div key={t.id} className={`toast ${t.kind}`}>{t.message}</div>)}</div>
    <div className='download-tray'>
      <h4>Task Center</h4>
      {!orderedTasks.length && <small>No queued background tasks</small>}
      {orderedTasks.map((t)=><div key={t.id}><b>{t.label}</b><small>{t.kind} · {t.status} · {t.step}</small><progress max={1} value={Math.max(0,Math.min(1,t.progress))}/></div>)}
    </div>
  </TaskCenterCtx.Provider></ToastCtx.Provider>;
}

export function EmptyState({title,action}:{title:string;action?:ReactNode}){ return <div className='empty'><p>{title}</p>{action}</div>; }

export class PageErrorBoundary extends React.Component<{children:ReactNode},{hasError:boolean}>{
  constructor(props:{children:ReactNode}){ super(props); this.state={hasError:false}; }
  static getDerivedStateFromError(){ return {hasError:true}; }
  componentDidCatch(err:unknown){ console.error(err); }
  render(){ if(this.state.hasError) return <EmptyState title='This section crashed. Please reload.' />; return this.props.children; }
}

export function GlobalShortcuts(){
  const nav = useNavigate();
  useEffect(()=>{ const onKey=(e:KeyboardEvent)=>{ if((e.ctrlKey||e.metaKey)&&e.key===','){ e.preventDefault(); nav('/settings'); }
    if((e.ctrlKey||e.metaKey)&&e.key.toLowerCase()==='n'){ e.preventDefault(); nav('/library'); }
  }; window.addEventListener('keydown',onKey); return ()=>window.removeEventListener('keydown',onKey); },[nav]);
  return null;
}

export function OfflineBanner({offline}:{offline:boolean}){ if(!offline) return null; return <div className='offline-banner'>Offline — showing cached results</div>; }
