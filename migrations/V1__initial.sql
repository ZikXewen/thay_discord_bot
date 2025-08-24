CREATE TYPE riot_region AS ENUM ('NA', 'BR', 'LAN', 'LAS', 'KR', 'JP', 'OCE', 'SG2', 'TW2', 'VN2', 'EUNE', 'EUW', 'ME1', 'TR', 'RU');
CREATE TABLE riot_auth (
  uid BIGINT PRIMARY KEY,
  region riot_region NOT NULL,
  riot_id VARCHAR(50) NOT NULL,
  puuid VARCHAR(78) NOT NULL
);
