import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DifyCredentialForm } from './DifyCredentialForm';
import { useCredentialStore } from '@/stores/useCredentialStore';

/**
 * DifyCredentialForm 组件测试
 * 覆盖组件渲染、用户交互和反馈机制
 */
describe('DifyCredentialForm 组件', () => {
  // 每个测试前重置 Store
  beforeEach(() => {
    useCredentialStore.getState().clearDifyCredential();
  });

  describe('初始渲染', () => {
    it('应正确渲染标题和表单字段', () => {
      render(<DifyCredentialForm />);
      
      expect(screen.getByText('Dify 工作流凭证')).toBeInTheDocument();
      expect(screen.getByLabelText('API 地址')).toBeInTheDocument();
      expect(screen.getByLabelText('API Key')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
    });

    it('初始状态应显示"未配置"徽章', () => {
      render(<DifyCredentialForm />);
      
      expect(screen.getByText('未配置')).toBeInTheDocument();
    });
  });

  describe('表单验证', () => {
    it('空字段失焦时应显示错误提示', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      // 聚焦再失焦 baseUrl
      const baseUrlInput = screen.getByLabelText('API 地址');
      await user.click(baseUrlInput);
      await user.tab();
      
      expect(screen.getByText('API 地址不能为空')).toBeInTheDocument();
    });

    it('无效 URL 应显示格式错误', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.type(screen.getByLabelText('API 地址'), 'invalid-url');
      await user.tab();
      
      expect(screen.getByText('请输入有效的 HTTP/HTTPS 地址')).toBeInTheDocument();
    });

    it('提交无效表单应显示顶部错误反馈', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.type(screen.getByLabelText('API 地址'), 'invalid');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      // 等待异步操作完成后检查反馈消息
      await waitFor(() => {
        const feedbackMessage = screen.getByText('请修正下方标红字段后重试');
        expect(feedbackMessage).toBeInTheDocument();
        expect(feedbackMessage.closest('[aria-live="assertive"]')).toBeInTheDocument();
      });
    });
  });

  describe('成功提交', () => {
    it('有效凭证提交后应显示成功反馈', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.type(screen.getByLabelText('API 地址'), 'https://api.dify.ai');
      await user.type(screen.getByLabelText('API Key'), 'app-test-key');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      await waitFor(() => {
        expect(screen.getByRole('status')).toHaveTextContent('已保存，待测试连接');
      });
    });

    it('提交后徽章应变为"已填写，待测试"', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.type(screen.getByLabelText('API 地址'), 'https://api.dify.ai');
      await user.type(screen.getByLabelText('API Key'), 'app-test-key');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      await waitFor(() => {
        expect(screen.getByText('已填写，待测试')).toBeInTheDocument();
      });
    });

    it('清空字段后提交应显示清空成功反馈', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      // 先填写并保存
      await user.type(screen.getByLabelText('API 地址'), 'https://api.dify.ai');
      await user.type(screen.getByLabelText('API Key'), 'app-test-key');
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      // 等待提交完成（按钮恢复为"保存"）
      await waitFor(() => {
        expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument();
      });
      
      // 清空字段
      await user.clear(screen.getByLabelText('API 地址'));
      await user.clear(screen.getByLabelText('API Key'));
      await user.click(screen.getByRole('button', { name: '保存' }));
      
      await waitFor(() => {
        expect(screen.getByRole('status')).toHaveTextContent('凭证已清空');
      });
    });
  });

  describe('Store 同步', () => {
    it('输入时 Store 应实时更新', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.type(screen.getByLabelText('API 地址'), 'https://test.dify.ai');
      
      const store = useCredentialStore.getState();
      expect(store.dify.baseUrl).toBe('https://test.dify.ai');
    });
  });

  describe('无障碍性', () => {
    it('错误字段应有正确的 aria-invalid 属性', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.click(screen.getByLabelText('API 地址'));
      await user.tab();
      
      expect(screen.getByLabelText('API 地址')).toHaveAttribute('aria-invalid', 'true');
    });

    it('错误消息应关联到对应输入框', async () => {
      const user = userEvent.setup();
      render(<DifyCredentialForm />);
      
      await user.click(screen.getByLabelText('API 地址'));
      await user.tab();
      
      const input = screen.getByLabelText('API 地址');
      const errorId = input.getAttribute('aria-describedby');
      expect(errorId).toBe('dify-base-url-error');
      expect(document.getElementById(errorId!)).toHaveTextContent('API 地址不能为空');
    });
  });
});
