
import React, { useState } from 'react';
import './App.css';

const mockLeaderboard = [
  { username: 'Alice', score: 1200, category: 'Logic', difficulty: 'Easy', region: 'NA' },
  { username: 'Bob', score: 1100, category: 'Math', difficulty: 'Medium', region: 'EU' },
  { username: 'Carol', score: 1050, category: 'Logic', difficulty: 'Hard', region: 'AS' },
  { username: 'Dave', score: 990, category: 'Math', difficulty: 'Easy', region: 'NA' },
];

const categories = ['All', 'Logic', 'Math'];
const difficulties = ['All', 'Easy', 'Medium', 'Hard'];
const regions = ['All', 'NA', 'EU', 'AS'];
const periods = ['All-time', 'Daily', 'Weekly', 'Monthly'];

function App() {
  const [category, setCategory] = useState('All');
  const [difficulty, setDifficulty] = useState('All');
  const [region, setRegion] = useState('All');
  const [period, setPeriod] = useState('All-time');

  const filtered = mockLeaderboard.filter(entry =>
    (category === 'All' || entry.category === category) &&
    (difficulty === 'All' || entry.difficulty === difficulty) &&
    (region === 'All' || entry.region === region)
  );

  return (
    <div className="leaderboard-container">
      <h1 className="leaderboard-title">Global Leaderboard</h1>
      <div className="filters">
        <select value={category} onChange={e => setCategory(e.target.value)}>
          {categories.map(c => <option key={c}>{c}</option>)}
        </select>
        <select value={difficulty} onChange={e => setDifficulty(e.target.value)}>
          {difficulties.map(d => <option key={d}>{d}</option>)}
        </select>
        <select value={region} onChange={e => setRegion(e.target.value)}>
          {regions.map(r => <option key={r}>{r}</option>)}
        </select>
        <select value={period} onChange={e => setPeriod(e.target.value)}>
          {periods.map(p => <option key={p}>{p}</option>)}
        </select>
      </div>
      <table className="leaderboard-table">
        <thead>
          <tr>
            <th>Rank</th>
            <th>Username</th>
            <th>Score</th>
            <th>Category</th>
            <th>Difficulty</th>
            <th>Region</th>
          </tr>
        </thead>
        <tbody>
          {filtered.map((entry, i) => (
            <tr key={entry.username} className="fade-in">
              <td>{i + 1}</td>
              <td>{entry.username}</td>
              <td>{entry.score}</td>
              <td>{entry.category}</td>
              <td>{entry.difficulty}</td>
              <td>{entry.region}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default App;
