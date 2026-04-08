/* [074A-7] Sección "Contenido" del panel admin — CMS editorial.
 * Sub-tabs laterales: Servicios | Blog | Proyectos | Equipo.
 * Cada sub-tab renderiza su propio editor/lista.
 * [074A-9] Sub-tab Servicios conectada a ListaServicios + EditorServicio.
 * [074A-11] Sub-tab Blog conectada a ListaBlog + EditorBlog.
 * [074A-12] Sub-tab Proyectos conectada a ListaProyectos + EditorProyecto.
 * [074A-13] Sub-tab Equipo conectada a ListaEquipo + EditorMiembro.
 * sentinel-disable-file componente-sin-hook limite-useState limite-lineas: Orquestador de sub-tabs CMS.
 * Los useState restantes son UI state (editor abierto/cerrado, item seleccionado)
 * que no justifican un hook porque son triviales y específicos del orquestador. */
import React, {useState, useCallback, useEffect} from 'react';
import {Briefcase, PenTool, FolderOpen, Users} from 'lucide-react';
import {Button} from '../ui/Button';
import {ListaServicios} from './ListaServicios';
import {EditorServicio} from './EditorServicio';
import {ListaBlog} from './ListaBlog';
import {EditorBlog} from './EditorBlog';
import {ListaProyectos} from './ListaProyectos';
import {EditorProyecto} from './EditorProyecto';
import {ListaEquipo} from './ListaEquipo';
import {EditorMiembro} from './EditorMiembro';
import {useContenidoServicios} from '../../hooks/useContenidoServicios';
import {useContenidoBlog} from '../../hooks/useContenidoBlog';
import {useContenidoProyectos} from '../../hooks/useContenidoProyectos';
import {useContenidoEquipo} from '../../hooks/useContenidoEquipo';
import type {AdminService, CreateServiceBody, UpdateServiceBody, SavePlanBody} from '../../api/admin-services';
import {apiSaveServicePlans} from '../../api/admin-services';
import type {AdminBlogPost, CreateBlogPostBody, UpdateBlogPostBody} from '../../api/admin-blog';
import type {AdminProject, CreateProjectBody, UpdateProjectBody} from '../../api/admin-projects';
import type {AdminTeamMember, CreateTeamMemberBody, UpdateTeamMemberBody} from '../../api/admin-team';
import './SeccionContenido.css';

type SubTab = 'servicios' | 'blog' | 'proyectos' | 'equipo';

interface SubTabConfig {
    id: SubTab;
    label: string;
    icono: React.ElementType;
}

const SUB_TABS: SubTabConfig[] = [
    {id: 'servicios', label: 'Servicios', icono: Briefcase},
    {id: 'blog', label: 'Blog', icono: PenTool},
    {id: 'proyectos', label: 'Proyectos', icono: FolderOpen},
    {id: 'equipo', label: 'Equipo', icono: Users},
];

/* [084A-9] Clave de sessionStorage para persistir sub-tab CMS entre recargas */
const CMS_SUBTAB_KEY = 'panel-cms-subtab';
const VALID_SUBTABS: SubTab[] = SUB_TABS.map(t => t.id);

export const SeccionContenido: React.FC = () => {
    /* [084A-9] Restaurar sub-tab desde sessionStorage si es válida */
    const [subTab, setSubTab] = useState<SubTab>(() => {
        const stored = sessionStorage.getItem(CMS_SUBTAB_KEY) as SubTab | null;
        if (stored && VALID_SUBTABS.includes(stored)) return stored;
        return 'servicios';
    });

    /* [084A-9] Persistir sub-tab en sessionStorage */
    useEffect(() => {
        sessionStorage.setItem(CMS_SUBTAB_KEY, subTab);
    }, [subTab]);

    /* [074A-9] Estado del CMS servicios */
    const {servicios, cargando, error, guardando, crear, actualizar, archivar} = useContenidoServicios();
    const [editorAbierto, setEditorAbierto] = useState(false);
    const [servicioEditando, setServicioEditando] = useState<AdminService | null>(null);

    const handleEditarServicio = useCallback((svc: AdminService) => {
        setServicioEditando(svc);
        setEditorAbierto(true);
    }, []);

    const handleCrearServicio = useCallback(() => {
        setServicioEditando(null);
        setEditorAbierto(true);
    }, []);

    const handleGuardarServicio = useCallback(async (body: CreateServiceBody | UpdateServiceBody, planes: SavePlanBody[]) => {
        if (servicioEditando) {
            const result = await actualizar(servicioEditando.id, body as UpdateServiceBody);
            if (result) {
                await apiSaveServicePlans(servicioEditando.id, planes);
                setEditorAbierto(false);
            }
        } else {
            const result = await crear(body as CreateServiceBody);
            if (result) {
                if (planes.length > 0) {
                    await apiSaveServicePlans(result.id, planes);
                }
                setEditorAbierto(false);
            }
        }
    }, [servicioEditando, actualizar, crear]);

    const handleArchivarServicio = useCallback(async (id: string) => {
        await archivar(id);
    }, [archivar]);

    /* [114A-7] Desarchivar = actualizar status a draft */
    const handleDesarchivarServicio = useCallback(async (id: string) => {
        await actualizar(id, {status: 'draft'} as UpdateServiceBody);
    }, [actualizar]);

    /* [074A-11] Estado del CMS blog */
    const {
        posts: blogPosts,
        cargando: blogCargando,
        error: blogError,
        guardando: blogGuardando,
        crear: blogCrear,
        actualizar: blogActualizar,
        archivar: blogArchivar,
    } = useContenidoBlog();
    const [blogEditorAbierto, setBlogEditorAbierto] = useState(false);
    const [postEditando, setPostEditando] = useState<AdminBlogPost | null>(null);

    const handleEditarPost = useCallback((post: AdminBlogPost) => {
        setPostEditando(post);
        setBlogEditorAbierto(true);
    }, []);

    const handleCrearPost = useCallback(() => {
        setPostEditando(null);
        setBlogEditorAbierto(true);
    }, []);

    const handleGuardarPost = useCallback(async (body: CreateBlogPostBody | UpdateBlogPostBody) => {
        if (postEditando) {
            const result = await blogActualizar(postEditando.id, body as UpdateBlogPostBody);
            if (result) setBlogEditorAbierto(false);
        } else {
            const result = await blogCrear(body as CreateBlogPostBody);
            if (result) setBlogEditorAbierto(false);
        }
    }, [postEditando, blogActualizar, blogCrear]);

    const handleArchivarPost = useCallback(async (id: string) => {
        await blogArchivar(id);
    }, [blogArchivar]);

    const handleDesarchivarPost = useCallback(async (id: string) => {
        await blogActualizar(id, {status: 'draft'} as UpdateBlogPostBody);
    }, [blogActualizar]);

    /* [074A-12] Estado del CMS proyectos */
    const {
        proyectos: proyectosList,
        cargando: proyectosCargando,
        error: proyectosError,
        guardando: proyectosGuardando,
        crear: proyectosCrear,
        actualizar: proyectosActualizar,
        archivar: proyectosArchivar,
    } = useContenidoProyectos();
    const [proyectoEditorAbierto, setProyectoEditorAbierto] = useState(false);
    const [proyectoEditando, setProyectoEditando] = useState<AdminProject | null>(null);

    const handleEditarProyecto = useCallback((proy: AdminProject) => {
        setProyectoEditando(proy);
        setProyectoEditorAbierto(true);
    }, []);

    const handleCrearProyecto = useCallback(() => {
        setProyectoEditando(null);
        setProyectoEditorAbierto(true);
    }, []);

    const handleGuardarProyecto = useCallback(async (body: CreateProjectBody | UpdateProjectBody) => {
        if (proyectoEditando) {
            const result = await proyectosActualizar(proyectoEditando.id, body as UpdateProjectBody);
            if (result) setProyectoEditorAbierto(false);
        } else {
            const result = await proyectosCrear(body as CreateProjectBody);
            if (result) setProyectoEditorAbierto(false);
        }
    }, [proyectoEditando, proyectosActualizar, proyectosCrear]);

    const handleArchivarProyecto = useCallback(async (id: string) => {
        await proyectosArchivar(id);
    }, [proyectosArchivar]);

    const handleDesarchivarProyecto = useCallback(async (id: string) => {
        await proyectosActualizar(id, {status: 'draft'} as UpdateProjectBody);
    }, [proyectosActualizar]);

    /* [074A-13] Estado del CMS equipo */
    const {
        miembros: equipoList,
        cargando: equipoCargando,
        error: equipoError,
        guardando: equipoGuardando,
        crear: equipoCrear,
        actualizar: equipoActualizar,
        archivar: equipoArchivar,
    } = useContenidoEquipo();
    const [miembroEditorAbierto, setMiembroEditorAbierto] = useState(false);
    const [miembroEditando, setMiembroEditando] = useState<AdminTeamMember | null>(null);

    const handleEditarMiembro = useCallback((m: AdminTeamMember) => {
        setMiembroEditando(m);
        setMiembroEditorAbierto(true);
    }, []);

    const handleCrearMiembro = useCallback(() => {
        setMiembroEditando(null);
        setMiembroEditorAbierto(true);
    }, []);

    const handleGuardarMiembro = useCallback(async (id: string | null, body: CreateTeamMemberBody | UpdateTeamMemberBody) => {
        if (id) {
            await equipoActualizar(id, body as UpdateTeamMemberBody);
        } else {
            await equipoCrear(body as CreateTeamMemberBody);
        }
        setMiembroEditorAbierto(false);
    }, [equipoActualizar, equipoCrear]);

    const handleArchivarMiembro = useCallback(async (id: string) => {
        await equipoArchivar(id);
    }, [equipoArchivar]);

    const handleDesarchivarMiembro = useCallback(async (id: string) => {
        await equipoActualizar(id, {status: 'draft'} as UpdateTeamMemberBody);
    }, [equipoActualizar]);

    return (
        <div className="contenidoContenedor">
            <div className="contenidoSubTabs">
                {SUB_TABS.map(tab => {
                    const Icono = tab.icono;
                    return (
                        <Button
                            key={tab.id}
                            type="button"
                            variante="texto"
                            className={`contenidoSubTab ${subTab === tab.id ? 'contenidoSubTab--activo' : ''}`}
                            onClick={() => setSubTab(tab.id)}
                        >
                            <Icono size={16} />
                            {tab.label}
                        </Button>
                    );
                })}
            </div>

            <div className="contenidoPanel">
                {subTab === 'servicios' && (
                    <>
                        {error && <div className="contenidoError">{error}</div>}
                        <ListaServicios
                            servicios={servicios}
                            cargando={cargando}
                            onEditar={handleEditarServicio}
                            onCrear={handleCrearServicio}
                            onArchivar={handleArchivarServicio}
                            onDesarchivar={handleDesarchivarServicio}
                        />
                        <EditorServicio
                            abierto={editorAbierto}
                            onCerrar={() => setEditorAbierto(false)}
                            servicio={servicioEditando}
                            onGuardar={handleGuardarServicio}
                            guardando={guardando}
                        />
                    </>
                )}
                {subTab === 'blog' && (
                    <>
                        {blogError && <div className="contenidoError">{blogError}</div>}
                        <ListaBlog
                            posts={blogPosts}
                            cargando={blogCargando}
                            onEditar={handleEditarPost}
                            onCrear={handleCrearPost}
                            onArchivar={handleArchivarPost}
                            onDesarchivar={handleDesarchivarPost}
                        />
                        <EditorBlog
                            abierto={blogEditorAbierto}
                            onCerrar={() => setBlogEditorAbierto(false)}
                            post={postEditando}
                            onGuardar={handleGuardarPost}
                            guardando={blogGuardando}
                        />
                    </>
                )}
                {subTab === 'proyectos' && (
                    <>
                        {proyectosError && <div className="contenidoError">{proyectosError}</div>}
                        <ListaProyectos
                            proyectos={proyectosList}
                            cargando={proyectosCargando}
                            onEditar={handleEditarProyecto}
                            onCrear={handleCrearProyecto}
                            onArchivar={handleArchivarProyecto}
                            onDesarchivar={handleDesarchivarProyecto}
                        />
                        <EditorProyecto
                            abierto={proyectoEditorAbierto}
                            onCerrar={() => setProyectoEditorAbierto(false)}
                            proyecto={proyectoEditando}
                            onGuardar={handleGuardarProyecto}
                            guardando={proyectosGuardando}
                        />
                    </>
                )}
                {subTab === 'equipo' && (
                    <>
                        {equipoError && <div className="contenidoError">{equipoError}</div>}
                        <ListaEquipo
                            miembros={equipoList}
                            cargando={equipoCargando}
                            onEditar={handleEditarMiembro}
                            onCrear={handleCrearMiembro}
                            onArchivar={handleArchivarMiembro}
                            onDesarchivar={handleDesarchivarMiembro}
                        />
                        <EditorMiembro
                            abierto={miembroEditorAbierto}
                            onCerrar={() => setMiembroEditorAbierto(false)}
                            miembro={miembroEditando}
                            onGuardar={handleGuardarMiembro}
                            guardando={equipoGuardando}
                        />
                    </>
                )}
            </div>
        </div>
    );
};
