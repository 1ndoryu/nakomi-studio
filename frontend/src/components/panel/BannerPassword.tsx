/* [154A-5] Banner que avisa al usuario de quick_register que establezca su contraseña.
 * Se muestra en la parte superior del panel cuando needsPassword = true.
 * Incluye formulario inline para establecer contraseña sin salir del panel. */
import {useBannerPassword} from '../../hooks/useBannerPassword';
import {Button} from '../ui/Button';
import {Input} from '../ui/Input';
import './BannerPassword.css';

export function BannerPassword() {
    const {
        needsPassword,
        password,
        setPassword,
        confirmar,
        setConfirmar,
        cargando,
        expandido,
        setExpandido,
        handleSubmit,
    } = useBannerPassword();

    if (!needsPassword) return null;

    return (
        <div className="bannerPassword">
            <div className="bannerPasswordContenido">
                <svg className="bannerPasswordIcono" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
                </svg>
                <p className="bannerPasswordTexto">
                    Tu cuenta no tiene contraseña. Establece una para poder iniciar sesión en el futuro.
                </p>
                {!expandido && (
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        onClick={() => setExpandido(true)}
                        className="bannerPasswordBoton"
                    >
                        Crear contraseña
                    </Button>
                )}
            </div>
            {expandido && (
                <form className="bannerPasswordForm" onSubmit={handleSubmit}>
                    <Input
                        type="password"
                        placeholder="Nueva contraseña (min 8 caracteres)"
                        value={password}
                        onChange={e => setPassword(e.target.value)}
                        minLength={8}
                        required
                        className="bannerPasswordInput"
                        autoComplete="new-password"
                    />
                    <Input
                        type="password"
                        placeholder="Confirmar contraseña"
                        value={confirmar}
                        onChange={e => setConfirmar(e.target.value)}
                        minLength={8}
                        required
                        className="bannerPasswordInput"
                        autoComplete="new-password"
                    />
                    <Button
                        variante="primario"
                        tamano="pequeno"
                        type="submit"
                        disabled={cargando}
                        className="bannerPasswordBoton"
                    >
                        {cargando ? 'Guardando...' : 'Guardar contraseña'}
                    </Button>
                </form>
            )}
        </div>
    );
}
