import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { getProject, getProjectVersions, type ModrinthProject, type ModrinthVersion } from '../hooks/api';
export function ProjectDetailPage(){const {id=''}=useParams(); const [project,setProject]=useState<ModrinthProject|null>(null); const [versions,setVersions]=useState<ModrinthVersion[]>([]);
useEffect(()=>{void getProject(id).then(setProject); void getProjectVersions(id).then(setVersions)},[id]);
if(!project) return <p>Loading...</p>;
return <section><h2>{project.title}</h2><p>{project.description}</p><h3>Versions</h3><ul className='cards'>{versions.map(v=><li key={v.id}><b>{v.version_number}</b><p>{v.loaders.join(', ')} · {v.game_versions.join(', ')}</p></li>)}</ul></section>}
