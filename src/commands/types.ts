export type RuleFile = {
  id: number;
  name: string;
  isDir: boolean;
  parentId?: number;
  createdAt: number;
  updatedAt: number;
  children: RuleFile[];
};

export type UpsertRuleFileReq = {
  id?: number;
  name: string;
  isDir: boolean;
  parentId?: number;
};


export type UpdateRuleContentReq = {
  id: number;
  content: string;
}