import { useEffect, useState } from 'react';
import { listInstances, searchProjects, type InstanceSummary, type ModrinthProject } from '../hooks/api';
export function HomePage(){
  const [instances,setInstances]=useState<InstanceSummary[]>([]); const [mods,setMods]=useState<ModrinthProject[]>([]); const [packs,setPacks]=useState<ModrinthProject[]>([]);
  useEffect(()=>{void listInstances().then(s=>setInstances(s.instances)); void searchProjects({query:'',projectType:'modpack',limit:4,offset:0,sort:'downloads'}).then(r=>setPacks(r.hits)); void searchProjects({query:'',projectType:'mod',limit:4,offset:0,sort:'downloads'}).then(r=>setMods(r.hits));},[]);
  return <section><h2>Welcome back</h2><h3>Recently played</h3><ul className='cards'>{instances.slice(0,3).map(i=><li key={i.id}><b>{i.name}</b><p>{i.mc_version} · {i.loader}</p></li>)}</ul><h3>Featured modpacks</h3><ul className='cards'>{packs.map(p=><li key={p.project_id}><b>{p.title}</b><p>{p.description}</p></li>)}</ul><h3>Featured mods</h3><ul className='cards'>{mods.map(p=><li key={p.project_id}><b>{p.title}</b><p>{p.description}</p></li>)}</ul></section>
}
