export type RuleDir = {
  id: number;
  name: string;
  parentId?: number;
  createdAt: number;
  updatedAt: number;
  dirs: RuleDir[];
};

export type UpsertRuleDirReq = {
  id?: number;
  name: string;
  parentId?: number;
};
