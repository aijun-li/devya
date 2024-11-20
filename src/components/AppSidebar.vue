<script setup lang="ts">
import { ArrowDownUp, NotebookTabs } from 'lucide-vue-next';
import { RouterLink } from 'vue-router';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from './ui/sidebar';

const menuItems = [
  { title: 'Network', link: '/network', icon: ArrowDownUp },
  { title: 'Rules', link: '/rules', icon: NotebookTabs },
];
</script>

<template>
  <Sidebar>
    <SidebarHeader />
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem v-for="item in menuItems" :key="item.title">
              <SidebarMenuButton as-child>
                <RouterLink :to="item.link">
                  <component :is="item.icon" />
                  {{ item.title }}
                </RouterLink>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>
    <SidebarFooter />
  </Sidebar>
</template>
