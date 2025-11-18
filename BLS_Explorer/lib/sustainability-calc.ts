const S19_HASHRATE_TH = 110;
const S19_POWER_KW = 3.25;
const GRID_EMISSION_FACTOR = 0.5;

export interface NetworkImpact {
  estimatedMiners: number;
  totalPowerGW: number;
  dailyEnergyKWh: number;
  dailyCarbonKg: number;
  energyPerTxWh: number;
  carbonPerTxG: number;
}

export function calculateNetworkImpact(
  networkHashrateTH: number,
  txCount24h: number
): NetworkImpact {
  const estimatedMiners = networkHashrateTH / S19_HASHRATE_TH;

  const totalPowerKW = estimatedMiners * S19_POWER_KW;
  const totalPowerGW = totalPowerKW / 1_000_000;

  const dailyEnergyKWh = totalPowerKW * 24;

  const dailyCarbonKg = dailyEnergyKWh * GRID_EMISSION_FACTOR;

  const energyPerTxWh = (dailyEnergyKWh * 1000) / txCount24h;
  const carbonPerTxG = (dailyCarbonKg * 1000) / txCount24h;

  return {
    estimatedMiners: Math.floor(estimatedMiners),
    totalPowerGW,
    dailyEnergyKWh,
    dailyCarbonKg,
    energyPerTxWh,
    carbonPerTxG
  };
}

export function calculateNetworkGrade(energyPerTxWh: number): 'A+' | 'A' | 'B' | 'C' | 'D' {
  if (energyPerTxWh < 100) return 'A+';
  if (energyPerTxWh < 200) return 'A';
  if (energyPerTxWh < 500) return 'B';
  if (energyPerTxWh < 1000) return 'C';
  return 'D';
}

export function calculateTreesRequired(dailyCarbonKg: number): number {
  return Math.ceil(dailyCarbonKg / 20);
}

export function formatEnergyGWh(dailyEnergyKWh: number): string {
  const annualGWh = (dailyEnergyKWh * 365) / 1_000_000;
  return annualGWh.toFixed(2);
}
