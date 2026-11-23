export type VolumeRecord = {
  id: number;
  serial: string;
  guid: string;
  label: string;
  filesystem: string;
  lastDriveLetter: string | null;
  autoRemap: boolean;
  isOnline: boolean;
};

export type RemapCandidate = {
  volumeId: number;
  serial: string;
  guid: string;
  label: string;
  previousDriveLetter: string;
  currentDriveLetter: string;
};

export type VolumeSyncResult = {
  volumes: VolumeRecord[];
  remapCandidates: RemapCandidate[];
};

export type DataSourceRecord = {
  id: number;
  volumeId: number;
  rootPath: string;
  label: string | null;
  enabled: boolean;
  volumeLabel: string;
  driveLetter: string | null;
  volumeOnline: boolean;
};

export type MovieListItem = {
  id: number;
  title: string;
  year: number | null;
  resolutionWidth: number | null;
  resolutionHeight: number | null;
  driveLetter: string | null;
  status: "online" | "offline" | string;
  imdbId: string | null;
  tmdbId: string | null;
  relativePath: string;
  fileSize: number | null;
  videoCodec: string | null;
  hdr: boolean;
  duplicateGroupId: number | null;
};

export type LibraryCounts = {
  all: number;
  duplicates: number;
  fourK: number;
  hd1080: number;
  offline: number;
};

export type ScanProgress = {
  sourceIndex: number;
  sourceCount: number;
  sourceLabel: string;
  current: number;
  total: number;
  title: string;
};

export type ScanSummary = {
  moviesFound: number;
  moviesInserted: number;
  moviesUpdated: number;
  skipped: number;
  errors: string[];
};