import { useEffect, useState } from 'react';
import './AliceAvatar.css';

export type AliceAvatarState = 'reposo' | 'escribiendo' | 'pensando' | 'lista' | 'error' | 'saludo';
export type AliceExpression = 'neutral' | 'feliz' | 'seria' | 'sorprendida' | 'triste' | 'enojada';
type PortraitAsset = AliceExpression | 'ojos-cerrados';

export interface AliceAvatarProps {
  estado?: AliceAvatarState;
  size?: 'small' | 'brand' | 'hero';
  expression?: AliceExpression;
  className?: string;
  label?: string;
}

const expressions: PortraitAsset[] = ['neutral', 'feliz', 'seria', 'sorprendida', 'triste', 'enojada', 'ojos-cerrados'];
const stateExpressions: Record<AliceAvatarState, AliceExpression> = {
  reposo: 'neutral',
  escribiendo: 'seria',
  pensando: 'seria',
  lista: 'feliz',
  error: 'triste',
  saludo: 'feliz',
};

/** A presentation boundary: a future Live2D renderer can keep these same props. */
export function AliceAvatar({
  estado = 'reposo',
  size = 'brand',
  expression,
  className = '',
  label,
}: AliceAvatarProps) {
  const [loaded, setLoaded] = useState<Partial<Record<PortraitAsset, boolean>>>({});
  const [failed, setFailed] = useState<Partial<Record<PortraitAsset, boolean>>>({});
  const [blinking, setBlinking] = useState(false);
  const [greetingFinished, setGreetingFinished] = useState(false);
  const [reducedMotion, setReducedMotion] = useState(() =>
    typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches,
  );

  useEffect(() => {
    const media = window.matchMedia('(prefers-reduced-motion: reduce)');
    const updatePreference = () => setReducedMotion(media.matches);
    updatePreference();
    media.addEventListener('change', updatePreference);
    return () => media.removeEventListener('change', updatePreference);
  }, []);

  useEffect(() => {
    setGreetingFinished(false);
    if (estado !== 'saludo') return;
    const greetingTimer = window.setTimeout(() => setGreetingFinished(true), 1600);
    return () => window.clearTimeout(greetingTimer);
  }, [estado]);

  useEffect(() => {
    setBlinking(false);
    if (
      reducedMotion || estado !== 'reposo' || expression ||
      !loaded.neutral || failed.neutral ||
      !loaded['ojos-cerrados'] || failed['ojos-cerrados']
    ) return;

    let blinkEnd: number | undefined;
    // A dedicated closed-eye portrait keeps the happy expression separate from blinking.
    const blinkInterval = window.setInterval(() => {
      setBlinking(true);
      blinkEnd = window.setTimeout(() => setBlinking(false), 120);
    }, 11000);

    return () => {
      window.clearInterval(blinkInterval);
      window.clearTimeout(blinkEnd);
    };
  }, [estado, expression, reducedMotion, loaded.neutral, failed.neutral, loaded['ojos-cerrados'], failed['ojos-cerrados']]);

  const restingExpression = estado === 'saludo' && greetingFinished ? 'neutral' : stateExpressions[estado];
  const activeExpression: PortraitAsset = expression ?? (blinking ? 'ojos-cerrados' : restingExpression);
  const showFallback = !loaded[activeExpression] || !!failed[activeExpression];
  const accessibleLabel = label && (showFallback ? `${label}. Retrato no disponible` : label);

  return (
    <span
      className={`alice-avatar alice-avatar--${size}${blinking ? ' alice-avatar--blink' : ''} ${className}`.trim()}
      data-state={estado}
      data-expression={activeExpression}
      data-fallback={showFallback || undefined}
      role={label ? 'img' : undefined}
      aria-label={accessibleLabel}
      aria-hidden={label ? undefined : true}
    >
      <span className="alice-avatar__portrait">
        <svg
          className={`alice-avatar__fallback${showFallback ? ' is-visible' : ''}`}
          viewBox="0 0 120 144"
          fill="none"
          aria-hidden="true"
        >
          <ellipse cx="60" cy="69" rx="36" ry="46" stroke="currentColor" strokeWidth="1" opacity=".24" />
          <path d="M60 43c-4 11-18 12-18 25 0 7 5 12 10 15-8 0-18-5-21-12 0 13 12 24 29 30 17-6 29-17 29-30-3 7-13 12-21 12 5-3 10-8 10-15 0-13-14-14-18-25Z" stroke="currentColor" strokeWidth="1.5" />
          <path d="m60 61 11 14-11 17-11-17 11-14Z" fill="currentColor" opacity=".78" />
          <path d="m60 65 6 10-6 10-6-10 6-10Z" stroke="var(--alice-cream, #efe3c2)" strokeWidth=".8" />
          <path d="M60 31v6M60 106v6M23 68h5M92 68h5" stroke="currentColor" strokeLinecap="round" />
          <circle cx="60" cy="118" r="1.5" fill="currentColor" opacity=".6" />
        </svg>
        {expressions.map((portrait) => (
          <img
            key={portrait}
            className={`alice-avatar__image${portrait === activeExpression && !showFallback ? ' is-visible' : ''}`}
            src={`/assets/avatar/${portrait}.png`}
            alt=""
            aria-hidden="true"
            loading="eager"
            decoding="async"
            draggable={false}
            onLoad={() => setLoaded((current) => ({ ...current, [portrait]: true }))}
            onError={() => setFailed((current) => ({ ...current, [portrait]: true }))}
          />
        ))}
      </span>
      {estado === 'pensando' && (
        <span className="alice-avatar__thinking" aria-hidden="true">
          <i /><i /><i />
        </span>
      )}
    </span>
  );
}
