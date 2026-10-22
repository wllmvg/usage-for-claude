import { useEffect, useState } from 'react';

const reduced = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

export function useProgress(run, ms = 1400, delay = 0) {
  const [k, setK] = useState(0);
  useEffect(() => {
    if (!run) return;
    if (reduced()) return setK(1);
    let raf;
    let t0;
    const step = (t) => {
      t0 ??= t + delay;
      const x = Math.min(1, Math.max(0, (t - t0) / ms));
      setK(1 - Math.pow(1 - x, 4));
      if (x < 1) raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  }, [run, ms, delay]);
  return k;
}

export function useReveal() {
  useEffect(() => {
    const io = new IntersectionObserver(
      (entries) => entries.forEach((e) => {
        if (e.isIntersecting) {
          e.target.classList.add('in');
          io.unobserve(e.target);
        }
      }),
      { rootMargin: '0px 0px -12% 0px' },
    );
    document.querySelectorAll('.reveal').forEach((el) => io.observe(el));
    return () => io.disconnect();
  }, []);
}

export function useInView(ref) {
  const [seen, setSeen] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(([e]) => {
      if (e.isIntersecting) {
        setSeen(true);
        io.disconnect();
      }
    }, { threshold: 0.35 });
    io.observe(el);
    return () => io.disconnect();
  }, [ref]);
  return seen;
}

export function useScrolled(px = 8) {
  const [s, setS] = useState(false);
  useEffect(() => {
    const on = () => setS(scrollY > px);
    on();
    addEventListener('scroll', on, { passive: true });
    return () => removeEventListener('scroll', on);
  }, [px]);
  return s;
}
