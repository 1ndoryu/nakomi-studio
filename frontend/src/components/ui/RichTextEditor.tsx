/* [074A-7] Editor de texto enriquecido con tiptap.
 * Componente reutilizable: recibe contenido HTML inicial y notifica cambios.
 * Extensiones: StarterKit (bold, italic, heading, lists, etc), Image, Link, Placeholder.
 * Toolbar minimalista con las acciones más comunes.
 * sentinel-disable-file html-nativo-en-vez-de-componente: Los botones/inputs del toolbar usan
 * elementos nativos porque el componente <Button> añade clases (botonBase) que interfieren
 * con los estilos del toolbar. Mismo patrón que 074A-47 (headerPanelSubmenu). */
import React, {useCallback} from 'react';
import {useEditor, EditorContent} from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Image from '@tiptap/extension-image';
import Link from '@tiptap/extension-link';
import Placeholder from '@tiptap/extension-placeholder';
import {
    Bold, Italic, Heading1, Heading2, Heading3,
    List, ListOrdered, ImageIcon, LinkIcon, Undo, Redo, Code,
    Quote,
} from 'lucide-react';
import './RichTextEditor.css';

interface RichTextEditorProps {
    contenido: string;
    onChange: (html: string) => void;
    placeholder?: string;
}

export const RichTextEditor: React.FC<RichTextEditorProps> = ({
    contenido,
    onChange,
    placeholder = 'Escribe aquí...',
}) => {
    const editor = useEditor({
        extensions: [
            StarterKit.configure({
                heading: {levels: [1, 2, 3]},
            }),
            Image.configure({inline: false, allowBase64: false}),
            Link.configure({openOnClick: false, autolink: true}),
            Placeholder.configure({placeholder}),
        ],
        content: contenido,
        onUpdate: ({editor: ed}) => {
            onChange(ed.getHTML());
        },
    });

    const insertarImagen = useCallback(() => {
        if (!editor) return;
        const url = window.prompt('URL de la imagen:');
        if (url) {
            editor.chain().focus().setImage({src: url}).run();
        }
    }, [editor]);

    const insertarLink = useCallback(() => {
        if (!editor) return;
        const url = window.prompt('URL del enlace:');
        if (url) {
            editor.chain().focus().setLink({href: url}).run();
        }
    }, [editor]);

    if (!editor) return null;

    return (
        <div className="richTextEditor">
            <div className="richTextToolbar">
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('bold') ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleBold().run()}
                    title="Negrita"
                >
                    <Bold size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('italic') ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleItalic().run()}
                    title="Cursiva"
                >
                    <Italic size={16} />
                </button>
                <span className="richTextSeparador" />
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('heading', {level: 1}) ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleHeading({level: 1}).run()}
                    title="Título 1"
                >
                    <Heading1 size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('heading', {level: 2}) ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleHeading({level: 2}).run()}
                    title="Título 2"
                >
                    <Heading2 size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('heading', {level: 3}) ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleHeading({level: 3}).run()}
                    title="Título 3"
                >
                    <Heading3 size={16} />
                </button>
                <span className="richTextSeparador" />
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('bulletList') ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleBulletList().run()}
                    title="Lista"
                >
                    <List size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('orderedList') ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleOrderedList().run()}
                    title="Lista numerada"
                >
                    <ListOrdered size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('blockquote') ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleBlockquote().run()}
                    title="Cita"
                >
                    <Quote size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('codeBlock') ? 'richTextBtn--activo' : ''}`}
                    onClick={() => editor.chain().focus().toggleCodeBlock().run()}
                    title="Bloque de código"
                >
                    <Code size={16} />
                </button>
                <span className="richTextSeparador" />
                <button type="button" className="richTextBtn" onClick={insertarImagen} title="Imagen">
                    <ImageIcon size={16} />
                </button>
                <button
                    type="button"
                    className={`richTextBtn ${editor.isActive('link') ? 'richTextBtn--activo' : ''}`}
                    onClick={insertarLink}
                    title="Enlace"
                >
                    <LinkIcon size={16} />
                </button>
                <span className="richTextSeparador" />
                <button
                    type="button"
                    className="richTextBtn"
                    onClick={() => editor.chain().focus().undo().run()}
                    disabled={!editor.can().undo()}
                    title="Deshacer"
                >
                    <Undo size={16} />
                </button>
                <button
                    type="button"
                    className="richTextBtn"
                    onClick={() => editor.chain().focus().redo().run()}
                    disabled={!editor.can().redo()}
                    title="Rehacer"
                >
                    <Redo size={16} />
                </button>
            </div>
            <EditorContent editor={editor} className="richTextContenido" />
        </div>
    );
};
