import { useState, useEffect } from 'react';


/**
 * useEMA - A hook to calculate the Exponential Moving Average (EMA) of a value.
 *
 * @param {number} value The value to calculate the EMA for.
 * @param {number} alpha The alpha factor, between 0 and 1. Determines the weight of recent values.
 * @returns {number} The EMA of the given value.
 */
export function useEMA(value: number, alpha: number = 0.6): number {
    const [ema, setEma] = useState<number>(value);

    // Update the EMA whenever the input value changes
    useEffect(() => {
        const newEma = (1 - alpha) * ema + alpha * value;
        setEma(newEma);
    }, [alpha, ema, value]);

    return ema;
}
