import {useEffect, useMemo, useRef} from 'react';
import uPlot, {type AlignedData, type Options} from 'uplot';
import 'uplot/dist/uPlot.min.css';
import type {ResourceMetricPoint} from '../../api/hosting';
import './ResourceUsageChart.css';

function metricPercent(used: number | null, limit: number | null): number | null {
    if (used == null || limit == null || limit <= 0) return null;
    return Math.min(100, (used / limit) * 100);
}

function toChartData(points: ResourceMetricPoint[]): AlignedData {
    return [
        points.map(point => Math.floor(new Date(point.sampled_at).getTime() / 1000)),
        points.map(point => point.cpu_percent),
        points.map(point => metricPercent(point.ram_used_mb, point.ram_limit_mb)),
        points.map(point => metricPercent(point.disk_used_mb, point.disk_limit_mb)),
    ] as AlignedData;
}

function chartOptions(width: number, height: number): Options {
    const styles = getComputedStyle(document.documentElement);
    const textColor = styles.getPropertyValue('--text-muted').trim();
    const borderColor = styles.getPropertyValue('--border-default').trim();
    const cpuColor = styles.getPropertyValue('--color-info-text').trim() || styles.getPropertyValue('--brand-black').trim();
    const ramColor = styles.getPropertyValue('--color-warning-text').trim() || styles.getPropertyValue('--brand-black').trim();
    const diskColor = styles.getPropertyValue('--color-success-text').trim() || styles.getPropertyValue('--brand-black').trim();

    return {
        width,
        height,
        scales: {x: {time: true}, y: {range: [0, 100]}},
        axes: [
            {stroke: textColor, grid: {stroke: borderColor}},
            {stroke: textColor, grid: {stroke: borderColor}, values: (_plot, ticks) => ticks.map(value => `${value}%`)},
        ],
        series: [
            {},
            {label: 'CPU', stroke: cpuColor, width: 2},
            {label: 'RAM', stroke: ramColor, width: 2},
            {label: 'Disco', stroke: diskColor, width: 2},
        ],
        legend: {show: true},
    };
}

export function ResourceUsageChart({points}: {points: ResourceMetricPoint[]}) {
    const containerRef = useRef<HTMLDivElement>(null);
    const data = useMemo(() => toChartData(points), [points]);

    useEffect(() => {
        const container = containerRef.current;
        if (!container || points.length === 0) return undefined;

        let plot: uPlot | null = null;
        const render = () => {
            const width = Math.max(320, container.clientWidth);
            plot?.destroy();
            plot = new uPlot(chartOptions(width, 220), data, container);
        };

        const observer = new ResizeObserver(render);
        observer.observe(container);
        render();

        return () => {
            observer.disconnect();
            plot?.destroy();
        };
    }, [data, points.length]);

    if (points.length === 0) {
        return <div className="graficoRecursosVacio">Sin muestras todavía</div>;
    }

    return <div className="graficoRecursos" ref={containerRef} />;
}