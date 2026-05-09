/*
 * Componente: ModalTestimonio
 * Modal para que un visitante escriba un testimonio/comentario.
 * Campos: foto, nombre, puesto, servicio/proyecto relacionado, red social, comentario.
 * El testimonio queda pendiente de aprobación (no se muestra en tiempo real).
 * [044A-40] Refactorizado para usar componente <Modal> base. Sin botón X por diseño.
 */
import React, {useState} from 'react';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import {Textarea} from '../ui/Textarea';
import {Modal, ModalBody, ModalField, ModalLabel} from '../ui/Modal';
import './ModalTestimonio.css';

interface ModalTestimonioProps {
    abierto: boolean;
    onCerrar: () => void;
}

interface FormularioTestimonio {
    nombre: string;
    cargo: string;
    comentario: string;
    proyectoRelacionado: string;
    redSocial: string;
    foto: File | null;
}

const ESTADO_INICIAL: FormularioTestimonio = {
    nombre: '',
    cargo: '',
    comentario: '',
    proyectoRelacionado: '',
    redSocial: '',
    foto: null,
};

export const ModalTestimonio: React.FC<ModalTestimonioProps> = ({abierto, onCerrar}) => {
    const [formulario, setFormulario] = useState<FormularioTestimonio>(ESTADO_INICIAL);
    const [previewFoto, setPreviewFoto] = useState<string | null>(null);
    const [estado, setEstado] = useState<'idle' | 'enviando' | 'exito' | 'error'>('idle');

    const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
        const {name, value} = e.target;
        setFormulario(prev => ({...prev, [name]: value}));
    };

    const handleFoto = (e: React.ChangeEvent<HTMLInputElement>) => {
        const archivo = e.target.files?.[0] || null;
        setFormulario(prev => ({...prev, foto: archivo}));
        if (archivo) {
            const reader = new FileReader();
            reader.onloadend = () => setPreviewFoto(reader.result as string);
            reader.readAsDataURL(archivo);
        } else {
            setPreviewFoto(null);
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setEstado('enviando');

        try {
            /*
             * TO-DO: Enviar al backend via REST API.
             * Endpoint esperado: POST /wp-json/glory/v1/testimonios
             * Body: FormData con los campos del formulario.
             * El testimonio se guardará con estado "pendiente" en WP.
             */
            await new Promise(resolve => setTimeout(resolve, 1000));
            setEstado('exito');
            setFormulario(ESTADO_INICIAL);
            setPreviewFoto(null);
        } catch {
            setEstado('error');
        }
    };

    return (
        <Modal abierto={abierto} onCerrar={onCerrar} className="modalMedio modalTestimonioContenedor">
            {estado === 'exito' ? (
                    <div className="modalTestimonioExito">
                        <p className="modalTestimonioExitoParrafo">
                            Tu testimonio ha sido enviado y está pendiente de aprobación.
                            Lo revisaremos y publicaremos lo antes posible.
                        </p>
                        <Button variante="primario" onClick={onCerrar}>Cerrar</Button>
                    </div>
                ) : (
                    <>
                        <p className="modalTexto">Comparte tu experiencia trabajando con nosotros.</p>

                        <ModalBody as="form" onSubmit={handleSubmit}>
                            {/* Foto de perfil */}
                            <div className="testimonioFotoWrapper">
                                <label htmlFor="testimonioFoto" className="testimonioFotoLabel">
                                    {previewFoto ? (
                                        <img src={previewFoto} alt="Preview" className="testimonioFotoPreview" loading="lazy" />
                                    ) : (
                                        <div className="testimonioFotoPlaceholder">
                                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
                                                <path d="M12 5v14M5 12h14" />
                                            </svg>
                                            <span>Foto</span>
                                        </div>
                                    )}
                                </label>
                                <input
                                    type="file"
                                    id="testimonioFoto"
                                    accept="image/*"
                                    onChange={handleFoto}
                                    className="testimonioFotoInput"
                                />
                            </div>

                            <div className="testimonioGridCampos">
                                {/* Nombre */}
                                <ModalField>
                                    <ModalLabel htmlFor="testimonioNombre">Nombre completo *</ModalLabel>
                                    <Input
                                        type="text"
                                        id="testimonioNombre"
                                        name="nombre"
                                        value={formulario.nombre}
                                        onChange={handleChange}
                                        placeholder="Tu nombre"
                                        required
                                    />
                                </ModalField>

                                {/* Cargo / Puesto */}
                                <ModalField>
                                    <ModalLabel htmlFor="testimonioCargo">Puesto / Empresa</ModalLabel>
                                    <Input
                                        type="text"
                                        id="testimonioCargo"
                                        name="cargo"
                                        value={formulario.cargo}
                                        onChange={handleChange}
                                        placeholder="CEO, Empresa XYZ"
                                    />
                                </ModalField>

                                {/* Proyecto / Servicio relacionado */}
                                <ModalField>
                                    <ModalLabel htmlFor="testimonioProyecto">Servicio o proyecto relacionado</ModalLabel>
                                    <Input
                                        type="text"
                                        id="testimonioProyecto"
                                        name="proyectoRelacionado"
                                        value={formulario.proyectoRelacionado}
                                        onChange={handleChange}
                                        placeholder="Diseño Web, Branding, etc."
                                    />
                                </ModalField>

                                {/* Red social */}
                                <ModalField>
                                    <ModalLabel htmlFor="testimonioRedSocial">Red social (perfil)</ModalLabel>
                                    <Input
                                        type="url"
                                        id="testimonioRedSocial"
                                        name="redSocial"
                                        value={formulario.redSocial}
                                        onChange={handleChange}
                                        placeholder="https://linkedin.com/in/..."
                                    />
                                </ModalField>
                            </div>

                            {/* Comentario */}
                            <ModalField className="testimonioCampoCompleto">
                                <ModalLabel htmlFor="testimonioComentario">Tu comentario *</ModalLabel>
                                <Textarea
                                    id="testimonioComentario"
                                    name="comentario"
                                    value={formulario.comentario}
                                    onChange={handleChange}
                                    placeholder="Comparte tu experiencia..."
                                    rows={4}
                                    required
                                />
                            </ModalField>

                            <div className="testimonioFormZona">
                                <Button variante="primario" className="testimonioAccionEnviar" disabled={estado === 'enviando'}>
                                    {estado === 'enviando' ? 'Enviando...' : 'Enviar testimonio'}
                                </Button>
                                {estado === 'error' && (
                                    <p className="testimonioError">Hubo un error. Intenta de nuevo.</p>
                                )}
                                <p className="testimonioNota">
                                    Tu comentario será revisado antes de publicarse.
                                </p>
                            </div>
                        </ModalBody>
                    </>
                )}
        </Modal>
    );
};
