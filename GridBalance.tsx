import React from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import consumptionData from './data/mexicali-consumption.json';

const GridBalance = () => {
  const data = consumptionData.hourly;

  return (
    <div className="p-8 bg-zinc-950 min-h-screen text-white">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-4xl font-bold mb-2">Balance de Red - Mexicali</h1>
        <p className="text-emerald-400 mb-10">QuantumEnergyOS • Monitoreo en tiempo real</p>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Card de Cobre */}
          <div className="bg-zinc-900 p-8 rounded-2xl border border-amber-500/30">
            <div className="flex items-center gap-3 mb-6">
              <span className="text-3xl">🔴</span>
              <h2 className="text-2xl font-semibold text-amber-400">Estado del Cobre</h2>
            </div>
            <p className="text-5xl font-bold text-amber-400">87.4°C</p>
            <p className="text-sm text-gray-400 mt-1">Temperatura promedio de líneas</p>
            
            <div className="mt-8 text-xs uppercase tracking-widest text-gray-500">Pérdidas por calor</div>
            <p className="text-4xl font-bold text-red-400">4.8%</p>
            <p className="text-xs text-gray-500">de energía se pierde como calor en el cobre</p>
          </div>

          {/* Gráfica */}
          <div className="lg:col-span-2 bg-zinc-900 p-8 rounded-2xl">
            <h2 className="text-xl mb-6">Demanda Eléctrica por Hora</h2>
            <ResponsiveContainer width="100%" height={380}>
              <LineChart data={data}>
                <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                <XAxis dataKey="hour" stroke="#52525b" />
                <YAxis stroke="#52525b" />
                <Tooltip />
                <Line type="natural" dataKey="demand" stroke="#eab308" strokeWidth={4} dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div className="mt-6 text-center text-xs text-gray-500">
          El cobre es el corazón de la red. A más de 85°C, las pérdidas aumentan drásticamente.
        </div>
      </div>
    </div>
  );
};

export default GridBalance;
