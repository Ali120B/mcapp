import { useEffect, useState } from 'react';
import { createInstance, deleteInstance, detectJavaInstallations, downloadAdoptiumJava, listInstances, recommendJavaForMc, type InstanceSummary, type JavaInstallation, type JavaRecommendation } from '../hooks/api';

export function LibraryPage(){
  const [instances,setInstances]=useState<InstanceSummary[]>([]);
  const [name,setName]=useState('');
  const [mc,setMc]=useState('1.21.1');
  const [loader,setLoader]=useState('fabric');
  const [javas,setJavas]=useState<JavaInstallation[]>([]);
  const [rec,setRec]=useState<JavaRecommendation|null>(null);

  const refresh=()=>listInstances().then(s=>setInstances(s.instances));
  const refreshJava=()=>detectJavaInstallations().then(setJavas);

  useEffect(()=>{ void refresh(); void refreshJava(); },[]);

  return <section>
    <h2>Instance Library</h2>
    <div className='toolbar'>
      <input placeholder='Instance name' value={name} onChange={e=>setName(e.target.value)} />
      <input value={mc} onChange={e=>setMc(e.target.value)} />
      <input value={loader} onChange={e=>setLoader(e.target.value)} />
      <button onClick={()=>createInstance(name,mc,loader).then(s=>{setInstances(s.instances);setName('')})}>Create instance</button>
    </div>

    <h3>Java Management (Phase 7)</h3>
    <div className='toolbar'>
      <button onClick={()=>refreshJava()}>Detect Java</button>
      <button onClick={()=>recommendJavaForMc(mc).then(setRec)}>Match for MC {mc}</button>
      {rec && !rec.installed_match ? <button onClick={()=>downloadAdoptiumJava(rec.required_major).then(()=>refreshJava())}>Download Java {rec.required_major}</button> : null}
    </div>
    <ul className='cards'>
      {javas.map((j,idx)=><li key={idx}><b>Java {j.major}</b><p>{j.version}</p><small>{j.path}</small></li>)}
      {!javas.length && <li>No Java detected yet.</li>}
    </ul>
    {rec && <p>Required Java for {rec.mc_version}: <b>{rec.required_major}</b> {rec.installed_match ? '✅ installed' : '⚠️ missing'}</p>}

    <ul className='cards'>{instances.map(i=><li key={i.id}><b>{i.name}</b><p>{i.mc_version} · {i.loader}</p><button onClick={()=>deleteInstance(i.id).then(s=>setInstances(s.instances))}>Delete</button></li>)}</ul>
  </section>
}
