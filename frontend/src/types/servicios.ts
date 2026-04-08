import {Skill} from './contenido';

export interface Servicio {
    id: string;
    titulo: string;
    descripcion: string;
    imagen: string;
    categorias: string[];
    link: string;
    /* [084A-29] UUID original del backend para edición admin inline */
    adminId?: string;
    skills?: Skill[];
    cta?: {
        titulo: string;
        descripcion: string;
        precio: string;
        botones: {
            primario: string;
            secundario: string;
        };
    };
}
