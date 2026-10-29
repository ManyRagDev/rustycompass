import { normalizeColor } from './colors';
export function Badge(props: { color: string }) {
    return <span style={{color: normalizeColor(props.color)}}>Ready</span>;
}
